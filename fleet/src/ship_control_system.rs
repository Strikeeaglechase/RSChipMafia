use std::cell::RefCell;

use protologic_core::{
	missile_launcher::{
		missilelauncher_configure, missilelauncher_get_enginetype, missilelauncher_get_reloadtime, missilelauncher_get_warheadtype, missilelauncher_trigger,
		MissileEngineType, MissileWarheadType,
	},
	physics::vehicle_get_position,
	radar::RadarTargetType,
};

use crate::{
	controllers::{flight_controller::flight_set_target_point, radar_controller::get_radar_tracks, turret_controller::TurretController},
	datalink::{
		datalink::{dl_crunch_id, dl_net_id, send_message},
		messages::{intercept_task_assign::InterceptTaskAssign, message::Message},
	},
	math::{utils::now, vector3::Vector3},
};

#[derive(Clone, Copy, Debug)]
struct QueuedLaunch {
	cell: i32,
	times_at_zero: i32,
}

const MISSILE_LAUNCH_RATE: f32 = 1.0;
struct InterceptTask {
	contact_id: i64,
	#[allow(dead_code)]
	interceptor_id: u8,
	ring: u8,
}

pub struct ShipControlSystem {
	pub last_shot_time: f32,
	turrets: Vec<TurretController>,

	queued_launch_cells: Vec<QueuedLaunch>,
	last_missile_launch_time: f32,

	interceptors: Vec<u8>,
	intercept_tasks: Vec<InterceptTask>,

	has_started: bool,
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

			interceptors: Vec::new(),
			intercept_tasks: Vec::new(),

			has_started: false,
		}
	}

	fn init(&mut self) {
		if vehicle_get_position().2 > 0.0 {
			for _ in 0..1 {
				self.fire_missile(MissileWarheadType::Flak, MissileEngineType::HighThrust);
			}
		} else {
			for _ in 0..1 {
				self.fire_missile(MissileWarheadType::Nuclear, MissileEngineType::HighThrust);
			}
		}

		// self.fire_missile(0);
		// set_flight_mode(GuidanceMode::StopAtPoint);
		let side = vehicle_get_position().2.signum();
		flight_set_target_point(Vector3::new(350.0, 0.0, 500.0 * side));

		self.has_started = true;
	}

	fn update(&mut self) {
		if !self.has_started {
			return;
		}

		// println!(
		// 	"Free interceptors: {}, Intercept tasks: {}",
		// 	self.interceptors.len(),
		// 	self.intercept_tasks.len()
		// );

		if self.interceptors.len() > 0 {
			let tracks = get_radar_tracks();
			for track in tracks {
				if track.is_allied // Don't shoot down our missiles
					|| track.rc_type != RadarTargetType::Missile // Only shoot down missiles
					|| self.intercept_tasks.iter().any(|f| f.contact_id == track.id)
				// Don't shoot down the same missile twice
				{
					continue;
				}
				// let dist = (pos - track.get_current_position()).length();
				// if dist > 1000.0 {
				// 	continue;
				// }

				let mut next_free_ring = 0;
				for i in 0..15 {
					if !self.intercept_tasks.iter().any(|f| f.ring == i) {
						next_free_ring = i;
						break;
					}
				}

				let interceptor = self.interceptors.pop().unwrap();
				let message = InterceptTaskAssign::new(dl_net_id(track.id).unwrap(), dl_crunch_id(track.id), interceptor, next_free_ring);
				send_message(Message::InterceptTaskAssign(message));
				println!("Starting intercept with {} against {}", interceptor, track);
				self.intercept_tasks.push(InterceptTask {
					contact_id: track.id,
					interceptor_id: interceptor,
					ring: next_free_ring,
				});
			}
		}

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

	fn handle_dl_message(&mut self, message: Message) {
		match message {
			Message::InterceptTaskAssign(task) => {
				if task.contact_id == 0 && task.target_id == 0 {
					// Yippy we've got another interceptor to use!
					self.interceptors.push(task.interceptor_id);
				}
			}
			_ => {}
		}
	}

	fn check_queued_launches(&mut self) {
		let mut unfired_cells: Vec<QueuedLaunch> = Vec::new();

		// println!("Checking queued launches, currently {} queued", self.queued_launch_cells.len());
		for cell in self.queued_launch_cells.iter() {
			let maybe_unfired = self.maybe_fire_cell(cell);
			if let Some(unfired) = maybe_unfired {
				unfired_cells.push(unfired);
			} else {
				self.last_missile_launch_time = now();
			}
		}

		self.queued_launch_cells = unfired_cells;
	}

	fn maybe_fire_cell(&self, cell: &QueuedLaunch) -> Option<QueuedLaunch> {
		let reload_time = missilelauncher_get_reloadtime(cell.cell);

		if now() - self.last_missile_launch_time < MISSILE_LAUNCH_RATE {
			let new_times_at_zero = if reload_time == 0.0 { 0 } else { cell.times_at_zero + 1 };
			return Some(QueuedLaunch {
				cell: cell.cell,
				times_at_zero: new_times_at_zero,
			});
		}

		if reload_time > 0.0 {
			return Some(QueuedLaunch { cell: cell.cell, times_at_zero: 0 });
		}

		if cell.times_at_zero > 1 {
			println!("Firing cell {}", cell.cell);
			missilelauncher_trigger(cell.cell);
			return None;
		}

		return Some(QueuedLaunch {
			cell: cell.cell,
			times_at_zero: cell.times_at_zero + 1,
		});
	}

	fn fire_missile(&mut self, warhead: MissileWarheadType, engine: MissileEngineType) {
		// Try to find an already loaded cell with a nuclear missile
		let cell = (0..18).find(|&i| {
			let matching_warhead = missilelauncher_get_warheadtype(i) == warhead;
			let matching_engine = missilelauncher_get_enginetype(i) == engine;
			let reload_time = missilelauncher_get_reloadtime(i);
			if !matching_warhead || !matching_engine || reload_time > 0.0 {
				return false;
			}

			// Make sure not queued by someone else
			let not_queued = !self.queued_launch_cells.iter().any(|f| f.cell == i);
			return not_queued;
		});

		if let Some(cell) = cell {
			self.queued_launch_cells.push(QueuedLaunch { cell, times_at_zero: 0 });
			println!("Queued missile launch for cell {}", cell);
		} else {
			// Find a non-queued cell to load
			let cell = (0..19).find(|&i| {
				let not_queued = !self.queued_launch_cells.iter().any(|f| f.cell == i);
				return not_queued;
			});

			if let Some(cell) = cell {
				missilelauncher_configure(cell, engine, warhead, 1.0);
				self.queued_launch_cells.push(QueuedLaunch { cell, times_at_zero: 0 });
				println!("Queued missile launch for cell {}", cell);
			} else {
				println!("No cells available to load missile!");
			}
		}
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

pub fn scs_handle_dl_message(message: Message) {
	SCS.with(|f| f.borrow_mut().handle_dl_message(message));
}
