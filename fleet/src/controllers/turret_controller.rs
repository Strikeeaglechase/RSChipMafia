use std::f32::consts::PI;

use protologic_core::{
	constants::turret_shell_speed,
	guns::{
		gun_get_bearing, gun_get_elevation, gun_get_magazine_reloadtime, gun_get_magazine_remaining, gun_get_magazine_type, gun_get_refiretime, gun_reload,
		gun_set_bearing, gun_set_elevation, gun_set_fuse, gun_trigger, AmmoType,
	},
	physics::{vehicle_get_orientation, vehicle_get_position, vehicle_get_velocity},
};

use crate::{
	get,
	math::{first_order_intercept::first_order_intercept, quaternion::*, utils::now, vector3::*},
};

use super::radar_controller::{radar_get_contact, RadarTrack};

pub struct TurretController {
	target_id: i64,
	index: i32,

	pub last_shot_time: f32,
}

const SHOT_INTERVAL: f32 = 0.5;

impl TurretController {
	pub fn new(index: i32) -> TurretController {
		println!("Setting up turret {}!", index);
		gun_reload(index, AmmoType::Flak);
		TurretController { target_id: 0, last_shot_time: 0.0, index }
	}

	pub fn update(&mut self, last_shot_time: f32) {
		gun_set_fuse(self.index, 0.1);
		if self.target_id == 0 {
			return;
		}

		let target = get!(radar_get_contact(self.target_id));
		let lead_point = first_order_intercept(
			vehicle_get_position().into(),
			vehicle_get_velocity().into(),
			turret_shell_speed(),
			target.position,
			target.velocity,
		);

		let angles = TurretController::get_pointing_angles_for_position(lead_point);

		let bearing_error = angles.bearing - gun_get_bearing(self.index);
		let elevation_error = angles.elevation - gun_get_elevation(self.index);
		let acceptable_error = 0.1f32;

		if bearing_error.abs() + elevation_error.abs() < acceptable_error && self.ready_to_fire() && now() - last_shot_time > SHOT_INTERVAL {
			gun_trigger(self.index);
			println!("Firing turret {}!", self.index);
			self.last_shot_time = now();
		}

		gun_set_bearing(self.index, angles.bearing);
		gun_set_elevation(self.index, angles.elevation);

		if gun_get_magazine_remaining(self.index) == 0 && gun_get_magazine_reloadtime(self.index) == 0.0 {
			gun_reload(self.index, AmmoType::ArmourPiercing);
		}
	}

	pub fn self_det(&self) -> bool {
		if gun_get_magazine_remaining(self.index) == 0 && gun_get_magazine_reloadtime(self.index) == 0.0 {
			gun_reload(self.index, AmmoType::Flak);
			return false;
		}

		if self.ready_to_fire() {
			gun_set_fuse(self.index, 0.001);
			gun_trigger(self.index);
			return true;
		}
		return false;
	}

	pub fn set_target(&mut self, target: &RadarTrack) {
		if target.id == self.target_id {
			return;
		}

		self.target_id = target.id;
		let current_ammo_type = gun_get_magazine_type(self.index);
		if current_ammo_type != AmmoType::ArmourPiercing {
			gun_reload(self.index, AmmoType::ArmourPiercing);
		}
	}

	fn ready_to_fire(&self) -> bool {
		gun_get_refiretime(self.index) == 0.0 && gun_get_magazine_reloadtime(self.index) == 0.0 && gun_get_magazine_remaining(self.index) > 0
	}

	fn get_pointing_angles_for_position(position: Vector3) -> Angles {
		let ship_pos: Vector3 = vehicle_get_position().into();
		let mut dir = (ship_pos - position).normalized();

		let orientation: Quaternion = vehicle_get_orientation().into();
		dir = orientation.invert() * dir;

		let turret_rotation = Quaternion::from_axis_angle(&AxisAngle {
			axis: Vector3::new(0.0, 0.0, 1.0),
			angle: PI / 2.0,
		})
		.normalized();
		dir = turret_rotation.invert() * dir;

		let angles = dir.angles();
		return angles;
	}
}
