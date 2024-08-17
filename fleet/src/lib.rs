use core::Core;
use std::env;

use controllers::flight_controller::VehicleType;
use math::utils::now;
use protologic_core::{physics::vehicle_get_position, wait::*};

extern crate protologic_core;
pub mod controllers;
pub mod core;
pub mod datalink;
pub mod math;
pub mod missile_control_system;
pub mod radar_scan_pattern;
pub mod ship_control_system;
pub mod updatable_debug;

#[no_mangle]
pub extern "C" fn tick() {
	// if vehicle_get_position().2 == -5000.0 {
	// 	engine_set_throttle(1.0);
	// 	wait_ticks(5000);
	// 	engine_set_throttle(0.0);
	// }

	// if vehicle_get_position().2 < 0.0 {
	// 	return;
	// }

	// Print all environment variables
	let mut v_type = VehicleType::Ship;
	for (k, v) in env::vars() {
		println!("ENV: {} = {}", k, v);
		if k == "Type" {
			v_type = match v.as_str() {
				"Ship" => VehicleType::Ship,
				"Missile" => VehicleType::Missile,
				_ => VehicleType::Missile,
			};
		}
	}

	let mut core = Core::new(v_type);
	let mut prev_time = now();

	println!("Pos: {:?}", vehicle_get_position());

	loop {
		let dt = now() - prev_time;
		prev_time = now();
		core.tick(dt);
		wait_tick();
	}
}
