use std::cell::RefCell;

use protologic_core::{
	maneuvering::{engine_get_fuel_amount, engine_set_throttle, wheel_set_torque},
	physics::{vehicle_get_angular_velocity, vehicle_get_orientation, vehicle_get_position, vehicle_get_velocity},
};

use crate::{
	math::{
		pid::PID,
		quaternion::Quaternion,
		utils::{deg, lerp},
		vector3::Vector3,
	},
	updatable_debug_line::UpdatableDebugLine,
};

pub enum GuidanceMode {
	Drift,
	Stop,
	StopAtPoint,
	Impact,
}

#[derive(Clone, Copy, PartialEq)]
pub enum VehicleType {
	Ship,
	Missile,
}

pub struct FlightController {
	pub target_point: Vector3,
	pub target_point_velocity: Vector3,

	pub max_target_speed: f32,
	current_target_speed: f32,
	min_target_speed: f32,
	stop_range_start: f32,
	stopping_zone: f32,

	last_pulse_tick: u64,
	current_tick: u64,
	do_engine_pulsing: bool,

	pub guidance_mode: GuidanceMode,

	pid_x: PID,
	pid_y: PID,
	pid_z: PID,

	using_non_pid_guidance: bool,
	max_throttle: f32,

	command_line: UpdatableDebugLine,
	target_line: UpdatableDebugLine,
}

impl FlightController {
	fn new() -> FlightController {
		FlightController {
			target_point: Vector3::zero(),
			target_point_velocity: Vector3::zero(),

			max_target_speed: 30.0,
			current_target_speed: 0.0,
			stop_range_start: 1500.0,
			min_target_speed: 1.0,
			stopping_zone: 250.0,

			last_pulse_tick: 0,
			current_tick: 0,
			do_engine_pulsing: false,

			pid_x: PID::new(3.55, 0.0, 34.08, 10.0),
			pid_y: PID::new(3.55, 0.0, 34.08, 10.0),
			pid_z: PID::new(3.55, 0.0, 34.08, 10.0),

			using_non_pid_guidance: false,
			max_throttle: 1.0,

			guidance_mode: GuidanceMode::Drift,

			command_line: UpdatableDebugLine::new(),
			target_line: UpdatableDebugLine::new(),
		}
	}

	fn setup_for_missile(&mut self) {
		self.pid_x = PID::new(15.95, 0.0, 30.82, 10.0);
		self.pid_y = PID::new(15.95, 0.0, 30.82, 10.0);
		self.pid_z = PID::new(15.95, 0.0, 30.82, 10.0);

		self.max_target_speed = 250.0;
		self.min_target_speed = 25.0;
		self.stop_range_start = 3000.0;
	}

	fn update(&mut self, dt: f32) {
		self.target_point = self.target_point_velocity * dt + self.target_point;
		self.using_non_pid_guidance = false;
		self.max_throttle = 1.0;

		let maybe_target_point = self.get_target_point();

		if maybe_target_point.is_none() {
			if self.using_non_pid_guidance {
				return;
			}

			wheel_set_torque(0.0, 0.0, 0.0);
			engine_set_throttle(0.0);
			return;
		}

		let target_point = maybe_target_point.unwrap();

		if self.current_tick % 100 == 0 {
			self.command_line.set_a(vehicle_get_position().into());
			self.command_line.set_b(target_point);

			self.target_line.set_a(vehicle_get_position().into());
			self.target_line.set_b(self.target_point);
			self.target_line.set_color(1.0, 0.0, 0.0);
		}

		let ship_pos: Vector3 = vehicle_get_position().into();
		let ship_orientation: Quaternion = vehicle_get_orientation().into();

		let target_dir = (target_point - ship_pos).normalized();
		let local_dir = ship_orientation.invert() * target_dir;
		let target_orientation = Quaternion::from_vectors(&Vector3::new(0.0, 0.0, -1.0), &local_dir);
		let angle_error = target_orientation.axis_angle();
		let mut angle_error_degrees = deg(angle_error.angle);

		if angle_error_degrees.is_nan() {
			angle_error_degrees = 0.0;
			println!("Angle error is NaN");
		}

		if angle_error_degrees < 0.1 {
			self.kill_angular_velocity();
		} else {
			let axis = angle_error.axis.normalized();
			let p_x = self.pid_x.update(axis.x * angle_error_degrees, dt);
			let p_y = self.pid_y.update(axis.y * angle_error_degrees, dt);
			let p_z = self.pid_z.update(axis.z * angle_error_degrees, dt);
			let max = p_x.max(p_y).max(p_z).max(1.0);

			let torque_vector = ship_orientation * (Vector3::new(p_x, p_y, p_z) / max);

			wheel_set_torque(torque_vector.x, torque_vector.y, torque_vector.z);
		}

		let fuel = engine_get_fuel_amount();
		if fuel < 0.1 {
			// println!("Out of fuel!");
			engine_set_throttle(0.0);
			return;
		}

		let mut wanted_throttle: f32 = 0.0;
		if angle_error_degrees < 25.0 {
			wanted_throttle = 1.0;
		}
		if self.current_tick % 500 < 250 || !self.do_engine_pulsing {
			engine_set_throttle(wanted_throttle.min(self.max_throttle));

			if wanted_throttle > 0.0 {
				self.last_pulse_tick = self.current_tick;
			}
		} else {
			engine_set_throttle(0.0);
		}

		if self.do_engine_pulsing {
			if self.current_tick - self.last_pulse_tick > 1000 {
				engine_set_throttle(1.0);
			}

			if self.current_tick - self.last_pulse_tick > 1250 {
				engine_set_throttle(0.0);
				self.last_pulse_tick = self.current_tick;
			}
		}

		self.current_tick += 1;
	}

	fn get_target_point(&mut self) -> Option<Vector3> {
		match self.guidance_mode {
			GuidanceMode::Drift => return None,
			GuidanceMode::Stop => return self.get_stop_target_point(),
			GuidanceMode::StopAtPoint => return self.get_vel_corrected_target_point_for_stop(),
			GuidanceMode::Impact => return self.get_vel_corrected_target_point_for_impact(),
		}
	}

	fn get_stop_target_point(&self) -> Option<Vector3> {
		let ship_pos: Vector3 = vehicle_get_position().into();
		let ship_vel: Vector3 = vehicle_get_velocity().into();

		Some(ship_pos + ship_vel.normalized() * 10000.0)
	}

	fn get_vel_corrected_target_point_for_impact(&self) -> Option<Vector3> {
		let ship_pos: Vector3 = vehicle_get_position().into();
		let ship_vel: Vector3 = vehicle_get_velocity().into();

		let wanted_vel = (self.target_point - ship_pos).normalized();
		let current_vel = (ship_vel - self.target_point_velocity).normalized();
		let delta = wanted_vel - current_vel;

		Some(self.target_point + delta * 20000.0)
	}

	fn get_vel_corrected_target_point_for_stop(&mut self) -> Option<Vector3> {
		let ship_pos: Vector3 = vehicle_get_position().into();
		let ship_vel: Vector3 = vehicle_get_velocity().into();

		let dist_to_tp = (self.target_point - ship_pos).length();
		let speed = ship_vel.length();

		if dist_to_tp < 150.0 && speed < 15.0 {
			self.using_non_pid_guidance = true;
			self.kill_angular_velocity();
			return None;
		}

		if dist_to_tp < self.stopping_zone {
			if speed < 1.0 {
				self.using_non_pid_guidance = true;
				self.kill_angular_velocity();
				engine_set_throttle(0.0);

				return None;
			}

			self.max_throttle = 0.25;

			return Some(ship_pos + ship_vel.normalized() * -1000.0);
		} else {
			self.max_throttle = 1.0;
		}

		let cur_target_speed_max = self.max_target_speed.min(speed * 2.0);
		self.current_target_speed = self
			.min_target_speed
			.max(lerp(0.0, cur_target_speed_max, (dist_to_tp / self.stop_range_start).clamp(0.0, 1.0)));

		// println!(
		// 	"Max target speed: {}, current target speed: {}, current speed: {}",
		// 	cur_target_speed_max, self.current_target_speed, speed
		// );

		let wanted_vel = (self.target_point - ship_pos).normalized() * self.current_target_speed;
		// println!("Wanted vel: {}", wanted_vel);
		let delta = wanted_vel - ship_vel;

		Some(self.target_point + delta * 10000.0)
	}

	fn kill_angular_velocity(&self) {
		let mut angular_vel: Vector3 = vehicle_get_angular_velocity().into();
		angular_vel *= -10.0;

		wheel_set_torque(angular_vel.x, angular_vel.y, angular_vel.z)
	}
}

thread_local! {
	static FC: RefCell<FlightController> = RefCell::new(FlightController::new());
}

pub fn set_flight_mode(mode: GuidanceMode) {
	FC.with(|rfc| {
		let mut fc = rfc.borrow_mut();
		fc.guidance_mode = mode;
	});
}

pub fn flight_set_target_point(point: Vector3) {
	FC.with(|rfc| rfc.borrow_mut().target_point = point);
}

pub fn flight_set_target_point_velocity(vel: Vector3) {
	FC.with(|rfc| {
		rfc.borrow_mut().target_point_velocity = vel;
	});
}

pub fn update_flight_controller(dt: f32) {
	FC.with(|rfc| rfc.borrow_mut().update(dt));
}

pub fn setup_flight_for_missile() {
	FC.with(|rfc| rfc.borrow_mut().setup_for_missile());
}

pub fn get_max_flight_target_speed() -> f32 {
	FC.with(|rfc| rfc.borrow().max_target_speed)
}

pub fn current_flight_target_point() -> Vector3 {
	FC.with(|rfc| rfc.borrow().target_point)
}
