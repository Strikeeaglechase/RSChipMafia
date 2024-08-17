use std::env;

use protologic_core::physics::vehicle_get_position;

use crate::{
	controllers::{flight_controller::*, radar_controller::radar_update},
	datalink::{
		datalink::{dl_declare_host, get_core_message_queue, own_dl_id, send_message, update_dl},
		messages::{iff_pos::IFFPosition, message::Message},
	},
	math::{utils::now, vector3::Vector3},
	missile_control_system::{init_mcs, mcs_handle_dl_message, update_mcs},
	ship_control_system::{init_scs, scs_handle_dl_message, update_scs},
};

pub struct Core {
	vehicle_type: VehicleType,
	last_iff_time: f32,
}

const IFF_RATE: f32 = 0.5; // Once per second

impl Core {
	pub fn new(v_type: VehicleType) -> Core {
		if v_type == VehicleType::Ship {
			dl_declare_host();
		}
		Core { vehicle_type: v_type, last_iff_time: 0.0 }
	}

	pub fn tick(&mut self, dt: f32) {
		update_dl();
		radar_update();

		update_flight_controller(dt);

		let messages = get_core_message_queue();
		for message in messages {
			self.handle_dl_message(message);
		}

		match self.vehicle_type {
			VehicleType::Ship => update_scs(),
			VehicleType::Missile => update_mcs(),
		}

		if now() - self.last_iff_time > IFF_RATE {
			self.last_iff_time = now();
			self.iff_broadcast();
		}
	}

	fn iff_broadcast(&self) {
		let pos: Vector3 = vehicle_get_position().into();
		let message = Message::IFFPosition(IFFPosition::new(pos, own_dl_id()));

		send_message(message);
	}

	fn handle_dl_message(&mut self, message: Message) {
		match self.vehicle_type {
			VehicleType::Ship => scs_handle_dl_message(message),
			VehicleType::Missile => mcs_handle_dl_message(message),
		}
	}

	// fn get_track(&self, track_id: u16) -> &DatalinkTrack {
	// 	self.tracks.iter().find(|f| f.track_id == track_id).unwrap()
	// }
}

pub fn configure_control_system() {
	let v_type_key = env::var("Type").unwrap();
	match v_type_key.as_str() {
		"Ship" => {
			println!("Setting up vehicle as ship");
			init_scs();
		}
		"Missile" => {
			println!("Setting up vehicle as missile");
			setup_flight_for_missile();
			init_mcs();
		}
		_ => panic!("Unknown vehicle type"),
	}
}
