use std::cell::RefCell;

use protologic_core::{misc::self_destruct, physics::vehicle_get_position};

use crate::{
	controllers::flight_controller::*,
	datalink::{
		datalink::{datalink_disconnect, get_dl_track, get_ship_tracks, get_tick, send_message},
		messages::{assign_attack_target::AssignAttackTarget, message::Message, ready_attack_time::ReadyAttackTime},
	},
	math::{utils::now, vector3::Vector3},
};

#[derive(Clone, Copy, PartialEq)]
enum MissilePhase {
	WaitingForTarget,
	WaitingForAttackTime,
	Attack,
}

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
}

impl MissileControlSystem {
	fn new() -> MissileControlSystem {
		MissileControlSystem {
			target_id: u16::MAX,
			attack_time: 0,
			phase: MissilePhase::WaitingForTarget,
			wait_point: Vector3::zero(),
			time_loitering: 0,

			last_distance_to_target: 0.0,
			armed: false,
			allow_retarget: false,
			fallback_target: Vector3::zero(),
		}
	}

	fn init(&mut self) {
		self.wait_point = self.produce_wait_point();
		// set_flight_mode(GuidanceMode::StopAtPoint);
		// flight_set_target_point(self.wait_point);
		// flight_set_target_point_velocity(Vector3::zero());

		if vehicle_get_position().2 > 0.0 {
			self.fallback_target = Vector3::new(0.0, 0.0, -5000.0);
		} else {
			self.fallback_target = Vector3::new(0.0, 0.0, 5000.0);
		}

		self.setup_first_strike_mission();
	}

	fn setup_first_strike_mission(&mut self) {
		self.phase = MissilePhase::WaitingForAttackTime;
		self.wait_point = self.produce_wait_point_around(self.fallback_target);
		self.allow_retarget = true;

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

	fn setup_attack_mission(&mut self, aat: AssignAttackTarget) {
		if self.target_id == aat.target_id {
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
		println!("Attack time is now set to {}", self.attack_time);
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
		if self.phase == MissilePhase::Attack {
			let target = get_dl_track(self.target_id);

			let mut target_position: Vector3;
			let mut target_velocity = Vector3::zero();
			// Stale data!
			if target.is_none() || now() - target.unwrap().last_update_timestamp > 5.0 {
				if !self.allow_retarget && target.is_none() {
					println!("Missile has no target, and is not allowed to retarget");
					target_position = self.fallback_target;
				} else {
					// Give up and use stale data
					if !self.allow_retarget {
						target_position = target.unwrap().position;
						target_velocity = target.unwrap().velocity;
					} else {
						// try to grab a ship from datalink
						// Filter ship tracks on own side
						let tracks = get_ship_tracks();
						let ships = tracks
							.iter()
							.filter(|t| {
								if self.fallback_target.z > 0.0 {
									return t.position.z > 0.0;
								} else {
									return t.position.z < 0.0;
								}
							})
							.collect::<Vec<_>>();

						if ships.len() > 0 {
							let ship = ships[0];
							if ship.position.length_sq() == 0.0 {
								println!("Ship has no position, using fallback target");
								target_position = self.fallback_target;
							} else {
								target_position = ship.position;
								target_velocity = ship.velocity;
								self.target_id = ship.track_id;
							}
						} else {
							println!("Using fallback target: {:?}", self.fallback_target);
							target_position = self.fallback_target;
						}
					}
				}
			} else {
				target_position = target.unwrap().position;
				target_velocity = target.unwrap().velocity;
			}

			if target_position.length_sq() == 0.0 {
				println!(
					"After target selection logic, target position is still zero! Using fallback target: {:?}",
					self.fallback_target
				);
				target_position = self.fallback_target;
			}

			set_flight_mode(GuidanceMode::Impact);

			let current_target_point = current_flight_target_point();
			if (current_target_point - vehicle_get_position().into()).length_sq() > 1.0 {
				flight_set_target_point(target_position);
				flight_set_target_point_velocity(target_velocity);
			}

			let distance_to_target = (target_position - vehicle_get_position().into()).length();
			if !self.armed && distance_to_target < self.last_distance_to_target {
				self.armed = true;
				println!("Armed!");
			}

			if self.armed && distance_to_target > self.last_distance_to_target && distance_to_target < 250.0 {
				println!("Detonating!");
				self_destruct();
			}

			if self.armed && distance_to_target < 100.0 {
				datalink_disconnect();
			}

			self.last_distance_to_target = distance_to_target;
		} else if self.attack_time > 0 && get_tick() > self.attack_time {
			println!("Switching to attack phase");
			self.phase = MissilePhase::Attack;
		}

		if self.phase == MissilePhase::WaitingForTarget {
			self.time_loitering += 1;
		}
	}

	fn handle_dl_message(&mut self, message: Message) {
		match message {
			Message::ReadyAttackTime(rat) => self.update_ready_attack_time(rat),
			Message::AssignAttackTarget(aat) => self.setup_attack_mission(aat),
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
