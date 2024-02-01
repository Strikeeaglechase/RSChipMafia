use crate::{
	controllers::{flight_controller::*, radar_controller::radar_update},
	datalink::{
		datalink::{dl_declare_host, get_core_message_queue, update_dl},
		messages::message::Message,
	},
	missile_control_system::{init_mcs, mcs_handle_dl_message, update_mcs},
	ship_control_system::{init_scs, update_scs},
};

pub struct Core {
	vehicle_type: VehicleType,
}

impl Core {
	pub fn new(v_type: VehicleType) -> Core {
		match v_type {
			VehicleType::Ship => {
				println!("Setting up vehicle as ship");
				dl_declare_host();
				init_scs();
			}
			VehicleType::Missile => {
				println!("Setting up vehicle as missile");
				setup_flight_for_missile();
				init_mcs();
			}
		}

		Core { vehicle_type: v_type }
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
	}

	fn handle_dl_message(&mut self, message: Message) {
		match self.vehicle_type {
			VehicleType::Ship => {}
			VehicleType::Missile => {
				mcs_handle_dl_message(message);
			}
		}
	}

	// fn get_track(&self, track_id: u16) -> &DatalinkTrack {
	// 	self.tracks.iter().find(|f| f.track_id == track_id).unwrap()
	// }
}
