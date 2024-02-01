use std::cell::RefCell;

use protologic_core::missile_launcher::{
	missilelauncher_configure, missilelauncher_get_reloadtime, missilelauncher_trigger, MissileEngineType, MissileWarheadType,
};

use crate::{controllers::turret_controller::TurretController, math::utils::now};

#[derive(Clone, Copy, Debug)]
struct QueuedLaunch {
	cell: i32,
	has_reload_time: bool,
}

const MISSILE_LAUNCH_RATE: f32 = 1.0;
pub struct ShipControlSystem {
	pub last_shot_time: f32,
	turrets: Vec<TurretController>,
	queued_launch_cells: Vec<QueuedLaunch>,
	last_missile_launch_time: f32,
}

impl ShipControlSystem {
	fn new() -> ShipControlSystem {
		ShipControlSystem {
			last_shot_time: 0.0,
			turrets: vec![
				TurretController::new(0),
				TurretController::new(1),
				TurretController::new(2),
				TurretController::new(3),
			],
			queued_launch_cells: Vec::new(),
			last_missile_launch_time: 0.0,
		}
	}

	fn init(&mut self) {
		for i in 0..18 {
			self.fire_missile(i);
		}
		// self.fire_missile(0);
		// set_flight_mode(GuidanceMode::Impact);
		// flight_set_target_point(Vector3::new(200.0, 0.0, 0.0));
		// flight_set_target_point_velocity(Vector3::new(10.0, 0.0, 10.0))
		// flight_set_target_point(Vector3::new(5000.0, 0.0, 2000.0))
	}

	fn update(&mut self) {
		// if let Some(nearest_ship) = radar.get_nearest_ship() {
		// 	self.turrets.iter_mut().for_each(|f| f.set_target(nearest_ship));
		// }

		// if get_tick() % 50 == 0 {
		// 	let ship = get_nearest_ship();
		// 	if let Some(ship) = ship {
		// 		let track_id: u16 = get!(dl_net_id(ship.id));
		// 		let info_packet = TrackInfo::new(track_id, dl_crunch_id(ship.id), ship.rc_type);
		// 		let pos_packet = TrackPosition::new(track_id, ship.position);
		// 		let vel_packet = TrackVelocity::new(track_id, ship.velocity);

		// 		send_message(Message::TrackInfo(info_packet));
		// 		send_message(Message::TrackPosition(pos_packet));
		// 		send_message(Message::TrackVelocity(vel_packet));

		// 		let aat_packet = AssignAttackTarget::new(track_id);
		// 		send_message(Message::AssignAttackTarget(aat_packet));
		// 	}
		// }

		self.check_queued_launches();

		// for i in 0..19 {
		// 	let rl_time = missilelauncher_get_reloadtime(i);
		// 	println!("Cell {} reload time: {}", i, rl_time);
		// }

		for i in 0..self.turrets.len() {
			let last_shot_time = self.last_shot_time();

			let turret = &mut self.turrets[i];
			turret.update(last_shot_time);
		}
	}

	fn check_queued_launches(&mut self) {
		let mut unfired_cells: Vec<QueuedLaunch> = Vec::new();

		let mut has_fired = false;
		// println!("Checking queued launches, currently {} queued", self.queued_launch_cells.len());
		for cell in self.queued_launch_cells.iter() {
			let reload_time = missilelauncher_get_reloadtime(cell.cell);
			if reload_time == 0.0 {
				if cell.has_reload_time && !has_fired && now() - self.last_missile_launch_time >= MISSILE_LAUNCH_RATE {
					println!("Firing cell {}", cell.cell);
					missilelauncher_trigger(cell.cell);
					has_fired = true;
					self.last_missile_launch_time = now();
				} else {
					unfired_cells.push(QueuedLaunch {
						cell: cell.cell,
						has_reload_time: cell.has_reload_time,
					});
				}
			} else {
				unfired_cells.push(QueuedLaunch { cell: cell.cell, has_reload_time: true });
			}
		}

		self.queued_launch_cells = unfired_cells;
	}

	fn fire_missile(&mut self, cell: i32) {
		if self.queued_launch_cells.iter().any(|f| f.cell == cell) {
			println!("Cell {} already queued for launch!", cell);
			return;
		}
		println!("Queueing cell {}, currently {} queued", cell, self.queued_launch_cells.len());
		self.queued_launch_cells.push(QueuedLaunch { cell, has_reload_time: false });
		missilelauncher_configure(cell, MissileEngineType::HighThrust, MissileWarheadType::Nuclear, 1.0);
	}

	fn last_shot_time(&self) -> f32 {
		self.turrets.iter().map(|f| f.last_shot_time).fold(0.0, |a, b| a.max(b))
	}
}

thread_local! {
	static SCS: RefCell<ShipControlSystem> = RefCell::new(ShipControlSystem::new());
}

pub fn init_scs() {
	SCS.with(|scs| scs.borrow_mut().init());
}

pub fn update_scs() {
	SCS.with(|scs| scs.borrow_mut().update());
}
