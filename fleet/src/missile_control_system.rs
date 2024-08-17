use std::{cell::RefCell, env};

use protologic_core::{
	missile_launcher::MissileWarheadType,
	physics::vehicle_get_position,
	warhead::{self_destruct, warhead_arm},
};

use crate::{
	controllers::{
		flight_controller::*,
		radar_controller::{set_radar_mode, RadarMode},
	},
	datalink::{
		datalink::{datalink_disconnect, get_dl_track, get_ship_pos_from_iff, get_ship_tracks, get_tick, own_dl_id, send_message, DatalinkTrack},
		messages::{assign_attack_target::AssignAttackTarget, intercept_task_assign::InterceptTaskAssign, message::Message, ready_attack_time::ReadyAttackTime},
	},
	get, get_err,
	math::{utils::now, vector3::Vector3},
	updatable_debug::UpdatableDebugLine,
};

#[derive(Clone, Copy, PartialEq, Debug)]
enum MissilePhase {
	None,
	WaitingForTarget,
	WaitingForAttackTime,
	Attack,

	InterceptWait,
	InterceptAttack,
}

const RING_RANGES: [f32; 15] = [
	// 150.0, 300.0, 500.0, 1000.0, 1500.0, 2000.0, 150.0, 300.0, 500.0, 1000.0, 1500.0, 2000.0, 150.0, 300.0, 500.0,
	1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0, 1000.0,
];

pub struct MissileControlSystem {
	pub target_id: u16,
	attack_time: u32,

	phase: MissilePhase,
	wait_point: Vector3,
	time_loitering: u32,

	last_distance_to_target: f32,
	armed: bool,

	allow_retarget: bool,
	fallback_target: Vector3,
	warhead_type: MissileWarheadType,
	intercept_ring: u8,
	intercept_target_line: UpdatableDebugLine,

	has_started: bool,
}

impl MissileControlSystem {
	fn new() -> MissileControlSystem {
		MissileControlSystem {
			target_id: u16::MAX,
			attack_time: 0,
			phase: MissilePhase::None,
			wait_point: Vector3::zero(),
			time_loitering: 0,

			last_distance_to_target: 0.0,
			armed: false,
			allow_retarget: false,
			fallback_target: Vector3::zero(),
			warhead_type: MissileWarheadType::Nuclear,
			intercept_target_line: UpdatableDebugLine::new(),

			intercept_ring: 0,
			has_started: false,
		}
	}

	fn init(&mut self) {
		let warhead_type = env::vars().find(|(k, _)| k == "WarheadType").unwrap().1;
		let warhead = match warhead_type.as_str() {
			"Nuclear" => MissileWarheadType::Nuclear,
			"Flak" => MissileWarheadType::Flak,
			"Jammer" => MissileWarheadType::Jammer,
			"Inert" => MissileWarheadType::Inert,
			_ => MissileWarheadType::Nuclear,
		};

		self.warhead_type = warhead;

		self.has_started = true;
	}

	fn after_dl_init(&mut self) {
		match self.warhead_type {
			MissileWarheadType::Nuclear => {
				self.wait_point = self.produce_wait_point();
				if vehicle_get_position().2 > 0.0 {
					self.fallback_target = Vector3::new(0.0, 0.0, -5000.0);
				} else {
					self.fallback_target = Vector3::new(0.0, 0.0, 5000.0);
				}
				self.setup_first_strike_mission();
				self.phase = MissilePhase::WaitingForAttackTime;
			}
			MissileWarheadType::Flak => {
				// Let ship know we're flak missile
				let message = InterceptTaskAssign::new(0, 0, own_dl_id(), 0);
				send_message(Message::InterceptTaskAssign(message));

				let cur_pos: Vector3 = vehicle_get_position().into();
				self.wait_point = Vector3::random_direction() * 100.0;
				flight_set_target_point(cur_pos);
				flight_set_target_point_velocity(Vector3::zero());
				set_flight_mode(GuidanceMode::StopAtPoint);
				self.phase = MissilePhase::InterceptWait;
			}
			_ => {}
		}
	}

	fn setup_first_strike_mission(&mut self) {
		self.wait_point = self.produce_wait_point_around(self.fallback_target);
		self.allow_retarget = true;

		flight_set_target_point(self.wait_point);
		flight_set_target_point_velocity(Vector3::zero());
		set_flight_mode(GuidanceMode::StopAtPoint);

		// Declare time ready to attack
		let seconds_to_point = (self.wait_point - vehicle_get_position().into()).length() / (get_max_flight_target_speed() * 1.0);
		self.attack_time = get_tick() + (seconds_to_point * 100.0).round() as u32;

		println!("Attack time is set to {}", self.attack_time);
		// Send rat to datalink
		let rat = ReadyAttackTime::new(self.attack_time);
		send_message(Message::ReadyAttackTime(rat));
	}

	fn setup_attack_mission(&mut self, aat: AssignAttackTarget) {
		if self.phase != MissilePhase::WaitingForTarget || self.target_id == aat.target_id {
			return;
		}

		let opt_target = get_dl_track(aat.target_id);
		if opt_target.is_none() || opt_target.unwrap().position.length_sq() == 0.0 {
			return;
		}
		let target = opt_target.unwrap();
		self.target_id = target.track_id;

		if self.time_loitering > 1000 {
			self.phase = MissilePhase::Attack;
			return;
		}

		// Setup guidance
		self.phase = MissilePhase::WaitingForAttackTime;
		self.wait_point = self.produce_wait_point_around(target.position);

		flight_set_target_point(self.wait_point);
		flight_set_target_point_velocity(Vector3::zero());
		set_flight_mode(GuidanceMode::StopAtPoint);

		// Declare time ready to attack
		let seconds_to_point = (self.wait_point - vehicle_get_position().into()).length() / (get_max_flight_target_speed() * 0.75);
		self.attack_time = get_tick() + (seconds_to_point * 100.0).round() as u32;

		// Send rat to datalink
		let rat = ReadyAttackTime::new(self.attack_time);
		send_message(Message::ReadyAttackTime(rat));
	}

	fn update_ready_attack_time(&mut self, rat: ReadyAttackTime) {
		self.attack_time = rat.time.max(self.attack_time);
		// println!("Attack time is now set to {}", self.attack_time);
	}

	fn produce_wait_point_around(&self, pos: Vector3) -> Vector3 {
		let mut wp = pos + Vector3::random_direction() * 4000.0;
		while wp.length() > 5500.0 {
			wp = pos + Vector3::random_direction() * 4000.0;
		}

		return wp;
	}

	fn produce_wait_point(&self) -> Vector3 {
		return Vector3::random_direction() * 3000.0;
	}

	fn update(&mut self) {
		if !self.has_started {
			return;
		}

		// println!("Current phase: {:?}", self.phase);
		match self.phase {
			MissilePhase::None => self.after_dl_init(),
			MissilePhase::WaitingForTarget => self.time_loitering += 1,
			MissilePhase::WaitingForAttackTime => {
				if self.attack_time > 0 && get_tick() > self.attack_time {
					println!("Switching to attack phase. Current tick {}, RAT: {}", get_tick(), self.attack_time);
					self.phase = MissilePhase::Attack;
				}
			}
			MissilePhase::Attack => {
				let (target_position, target_velocity) = self.resolve_target_params();

				if target_position.length_sq() == 0.0 {
					println!("After target selection logic, target position is still zero!");
					set_flight_mode(GuidanceMode::Drift);
					return;
				}

				set_flight_mode(GuidanceMode::Impact);

				let current_target_point = current_flight_target_point();
				if (current_target_point - vehicle_get_position().into()).length_sq() > 1.0 {
					flight_set_target_point(target_position);
					flight_set_target_point_velocity(target_velocity);
				}

				self.run_warhead_logic(target_position);
			}
			MissilePhase::InterceptWait => {
				// Update hold point based off where ship is
				let ship_pos = get!(get_ship_pos_from_iff());
				let wait_point = ship_pos + self.wait_point;
				flight_set_target_point(wait_point);
				flight_set_target_point_velocity(Vector3::zero());
			}
			MissilePhase::InterceptAttack => {
				let ship_pos = get_err!(get_ship_pos_from_iff(), "No ship position found via IFF!");
				let target = get_err!(get_dl_track(self.target_id), "Unable to resolve target {} for intercept attack", self.target_id);
				let target_pos = target.position + target.velocity * (now() - target.last_update_timestamp);

				let iat_dist = RING_RANGES[self.intercept_ring as usize];
				let iat_dir = (target_pos - ship_pos).normalized();
				let iat_point = ship_pos + iat_dir * iat_dist;
				flight_set_target_point(iat_point);
				set_flight_mode(GuidanceMode::FastStopAtPoint);

				self.run_warhead_logic(target_pos);

				self.intercept_target_line.set_a(vehicle_get_position().into());
				self.intercept_target_line.set_b(target_pos);
				self.intercept_target_line.set_color(255.0 / 255.0, 242.0 / 255.0, 0.0);
			}
		}
	}

	fn resolve_target_params(&self) -> (Vector3, Vector3) {
		let maybe_target = get_dl_track(self.target_id);
		if let Some(target) = maybe_target {
			if target.position.length_sq() != 0.0 {
				let cur_target_pos = target.position + target.velocity * (now() - target.last_update_timestamp);
				return (cur_target_pos, target.velocity);
			}
		}

		// No target, or target has no position
		if self.allow_retarget {
			let alt_target = self.find_ship_target_alternative();
			if let Some(t) = alt_target {
				let cur_target_pos = t.position + t.velocity * (now() - t.last_update_timestamp);
				return (cur_target_pos, t.velocity);
			}
		}

		// Fallback position?
		if self.fallback_target.length_sq() != 0.0 {
			return (self.fallback_target, Vector3::zero());
		}

		// :( unable to resolve target
		(Vector3::zero(), Vector3::zero())
	}

	fn find_ship_target_alternative(&self) -> Option<DatalinkTrack> {
		let tracks = get_ship_tracks();
		let valid_ship_track = tracks.iter().find(|t| !t.is_allied);
		return valid_ship_track.copied();
	}

	fn run_warhead_logic(&mut self, target_position: Vector3) {
		let distance_to_target = (target_position - vehicle_get_position().into()).length();
		if !self.armed && distance_to_target < self.last_distance_to_target {
			self.armed = true;
			warhead_arm();
			println!("Armed!");
		}

		match self.warhead_type {
			MissileWarheadType::Nuclear => {
				if self.armed && distance_to_target > self.last_distance_to_target && distance_to_target < 250.0 {
					println!("Nuclear warhead detonating at distance {}", distance_to_target);
					self_destruct();
				}
			}
			MissileWarheadType::Flak => {
				if self.armed && distance_to_target < 250.0 {
					println!("Flack warhead detonating at distance {} (proximity)", distance_to_target);
					self_destruct();
				}

				// if self.armed && distance_to_target > self.last_distance_to_target && distance_to_target < 500.0 {
				// 	println!("Flack warhead detonating at distance {} (closure)", distance_to_target);
				// 	self_destruct();
				// }
			}
			_ => {}
		}

		if self.armed && distance_to_target < 100.0 {
			datalink_disconnect();
		}

		self.last_distance_to_target = distance_to_target;
	}

	fn handle_intercept_task(&mut self, task: InterceptTaskAssign) {
		if self.phase != MissilePhase::InterceptWait || task.contact_id == 0 || task.interceptor_id != own_dl_id() {
			return;
		}
		println!(
			"Missile {} starting intercept task against {} at ring {}. Contact Id: {}",
			own_dl_id(),
			task.target_id,
			task.ring,
			task.contact_id
		);

		self.target_id = task.target_id;
		self.allow_retarget = false;
		self.intercept_ring = task.ring;
		set_radar_mode(RadarMode::STT(task.contact_id));
		self.phase = MissilePhase::InterceptAttack;
	}

	fn handle_dl_message(&mut self, message: Message) {
		match message {
			Message::ReadyAttackTime(rat) => self.update_ready_attack_time(rat),
			Message::AssignAttackTarget(aat) => self.setup_attack_mission(aat),
			Message::InterceptTaskAssign(task) => self.handle_intercept_task(task),
			_ => {}
		}
	}
}

thread_local! {
	static MCS: RefCell<MissileControlSystem> = RefCell::new(MissileControlSystem::new());
}

pub fn init_mcs() {
	MCS.with(|f| f.borrow_mut().init());
}

pub fn mcs_handle_dl_message(message: Message) {
	MCS.with(|f| f.borrow_mut().handle_dl_message(message));
}

pub fn update_mcs() {
	MCS.with(|f| f.borrow_mut().update());
}
