use protologic_core::{
	physics::{vehicle_get_orientation, vehicle_get_position},
	radar::*,
};
use rand::seq::IteratorRandom;

use crate::{
	datalink::{
		datalink::{dl_crunch_id, dl_net_id, send_message},
		messages::{message::Message, track_info::TrackInfo, track_position::TrackPosition, track_velocity::TrackVelocity},
	},
	get,
	math::{
		quaternion::{AxisAngle, Quaternion},
		utils::{lerp, now},
		vector3::Vector3,
	},
	radar_scan_pattern::RadarScanPattern,
};
use std::{cell::RefCell, collections::HashMap, f32::consts::PI};

fn ctn(contact_type: RadarTargetType) -> &'static str {
	match contact_type {
		RadarTargetType::APShell => "APShell",
		RadarTargetType::FlakShell => "HEShell",
		RadarTargetType::Missile => "Missile",
		RadarTargetType::Asteroid => "Asteroid",
		RadarTargetType::SpaceBattleShip => "Ship",
		RadarTargetType::SpaceHulk => "SpaceHulk",
		RadarTargetType::Invalid => "Invalid",
	}
}

pub enum RadarMode {
	STT(i64),
	TWS,
	RWS,
}

const TWS_UPDATE_INTERVAL: f32 = 0.25; // 4 times per second
const TWS_MAX_AGE: f32 = 5.0; // 5 seconds
const DL_UPDATE_RATE: f32 = 1.0; // 1 times per second

#[derive(Clone, Copy, Debug)]
pub struct RadarTrack {
	pub id: i64,
	pub rc_type: RadarTargetType,

	pub position: Vector3,
	pub velocity: Vector3,

	pub last_update_timestamp: f32,
}

impl RadarTrack {
	pub fn new(contact: &RadarGetContactInfo) -> RadarTrack {
		RadarTrack {
			id: contact.id,
			rc_type: contact.target_type,

			position: Vector3::new(contact.x, contact.y, contact.z),
			velocity: Vector3::zero(),

			last_update_timestamp: now(),
		}
	}

	pub fn update_contact(&mut self, contact: &RadarGetContactInfo) {
		let dt = now() - self.last_update_timestamp;
		let new_pos = Vector3::new(contact.x, contact.y, contact.z);

		self.velocity = (new_pos - self.position) / dt;
		self.position = new_pos;

		self.last_update_timestamp = now();
	}

	pub fn get_current_position(&self) -> Vector3 {
		let dt = now() - self.last_update_timestamp;
		return self.position + self.velocity * dt;
	}

	pub fn time_since_last_update(&self) -> f32 {
		now() - self.last_update_timestamp
	}

	pub fn dist(&self) -> f32 {
		let ship_pos: Vector3 = vehicle_get_position().into();
		(ship_pos - self.get_current_position()).length()
	}
}

impl std::fmt::Display for RadarTrack {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let id = self.id.to_string()[..4].to_string();
		// let dist = self.dist().to_stri;
		write!(f, "RT({}, {}, {:.2})", ctn(self.rc_type), id, self.dist() / 1000.0)
	}
}

pub struct RadarController {
	current_scan_index: usize,
	scan_pattern: RadarScanPattern,

	tracks: Vec<RadarTrack>,
	pub mode: RadarMode,

	tws_in_row: i32,
	dl_update_times: HashMap<i64, f32>,
}

impl RadarController {
	fn new() -> RadarController {
		RadarController {
			current_scan_index: 0,
			scan_pattern: RadarScanPattern::new(),
			tracks: Vec::new(),
			mode: RadarMode::TWS,
			tws_in_row: 0,
			dl_update_times: HashMap::new(),
			// tws_line: None,
		}
	}

	fn update(&mut self) {
		// Grab and update contacts seen last tick
		let mut contacts: Vec<RadarGetContactInfo> = Vec::new();
		radar_get_contacts(&mut contacts);
		contacts.iter().for_each(|f| self.update_contact(f));

		self.point_radar();

		radar_trigger();
	}

	fn point_radar(&mut self) {
		match self.mode {
			RadarMode::STT(stt_id) => self.point_radar_for_stt(stt_id),
			RadarMode::TWS => self.point_radar_for_tws(),
			RadarMode::RWS => self.point_radar_for_rws(),
		}
	}

	fn point_radar_for_tws(&mut self) {
		if self.tws_in_row >= 1 {
			self.point_radar_for_rws();
			self.tws_in_row = 0;
			return;
		}

		// Find tracks that need an update
		let tracks_to_update = self
			.tracks
			.iter()
			.filter(|t| t.time_since_last_update() > TWS_UPDATE_INTERVAL && t.time_since_last_update() < TWS_MAX_AGE);

		// Choose a random one
		let opt_track = tracks_to_update.choose(&mut rand::thread_rng());
		match opt_track {
			Some(track) => {
				self.point_radar_direction(Vector3::from(vehicle_get_position()) - track.get_current_position());

				radar_set_angle(2.0);
				self.tws_in_row += 1;
				// self.tws_line = Some(debug_line(Vector3::from(vehicle_get_position()), track.position, 0.0, 1.0, 1.0));
			}
			None => {
				self.point_radar_for_rws();
				self.tws_in_row = 0;
			}
		}
	}

	fn point_radar_for_stt(&mut self, stt_id: i64) {
		if let Some(track) = self.get_contact(stt_id) {
			let time = track.time_since_last_update();
			if time > 1.0 {
				self.point_radar_for_rws();
			} else {
				self.point_radar_direction(track.get_current_position() - vehicle_get_position().into());

				// Increase angle longer not detected
				let angle = lerp(0.0, 90.0, time);
				radar_set_angle(angle);
			}
		} else {
			self.point_radar_for_rws();
		}
	}

	fn point_radar_for_rws(&mut self) {
		let dir = self.scan_pattern.get_point(self.current_scan_index);
		self.point_radar_direction(dir);
		radar_set_angle(40.0);
		self.current_scan_index = (self.current_scan_index + 1) % self.scan_pattern.size;
	}

	fn point_radar_direction(&self, dir: Vector3) {
		let orientation: Quaternion = vehicle_get_orientation().into();
		let local_dir = orientation.invert() * dir;

		let turret_rotation = Quaternion::from_axis_angle(&AxisAngle {
			axis: Vector3::new(0.0, 0.0, 1.0),
			angle: PI / 2.0,
		})
		.normalized();

		let radar_dir = turret_rotation.invert() * local_dir;

		let angles = radar_dir.angles();

		radar_set_bearing(angles.bearing);
		radar_set_elevation(angles.elevation);
	}

	fn update_contact(&mut self, contact: &RadarGetContactInfo) {
		let ut: RadarTrack = if let Some(track) = self.get_mut_contact(contact.id) {
			track.update_contact(contact);
			track.clone()
		} else {
			let track = RadarTrack::new(contact);
			// println!("New contact: {}", track);
			self.tracks.push(track);
			track.clone()
		};

		self.maybe_update_dl_track(&ut);
	}

	fn maybe_update_dl_track(&mut self, track: &RadarTrack) {
		match track.rc_type {
			RadarTargetType::SpaceBattleShip /*| RadarTargetType::Missile*/ => {
				let last_update = self.dl_update_times.get(&track.id).unwrap_or(&0.0);
				if now() - last_update > DL_UPDATE_RATE {
					// println!(
					// 	"{}: Updating DL for {:?} at {}, previous update time {}",
					// 	own_dl_id(),
					// 	track.id,
					// 	now(),
					// 	last_update
					// );
					// radar_send_dl_update(track.id, track.position.x, track.position.y, track.position.z);
					let track_id: u16 = get!(dl_net_id(track.id));
					let info_packet = TrackInfo::new(track_id, dl_crunch_id(track.id), track.rc_type);
					let pos_packet = TrackPosition::new(track_id, track.position);
					let vel_packet = TrackVelocity::new(track_id, track.velocity);

					send_message(Message::TrackInfo(info_packet));
					send_message(Message::TrackPosition(pos_packet));
					send_message(Message::TrackVelocity(vel_packet));

					self.dl_update_times.insert(track.id, now());
				}
			}
			_ => {}
		}
	}

	fn get_mut_contact(&mut self, id: i64) -> Option<&mut RadarTrack> {
		self.tracks.iter_mut().find(|f| f.id == id)
	}

	fn get_contact(&self, id: i64) -> Option<&RadarTrack> {
		self.tracks.iter().find(|f| f.id == id)
	}

	pub fn get_nearest_ship(&self) -> Option<&RadarTrack> {
		self
			.tracks
			.iter()
			.filter(|f| f.rc_type == RadarTargetType::SpaceBattleShip)
			.min_by(|a, b| a.dist().partial_cmp(&b.dist()).unwrap())
	}
}

thread_local! {
	static RADAR: RefCell<RadarController> = RefCell::new(RadarController::new());
}

pub fn radar_update() {
	RADAR.with(|radar| radar.borrow_mut().update());
}

pub fn radar_get_contact(id: i64) -> Option<RadarTrack> {
	RADAR.with(|radar| radar.borrow().get_contact(id).map(|f| *f))
}

pub fn get_nearest_ship() -> Option<RadarTrack> {
	RADAR.with(|radar| radar.borrow().get_nearest_ship().map(|f| *f))
}

pub fn set_radar_mode(mode: RadarMode) {
	RADAR.with(|radar| radar.borrow_mut().mode = mode);
}
