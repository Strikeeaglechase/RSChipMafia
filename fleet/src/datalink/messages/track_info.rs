use protologic_core::radar::RadarTargetType;

use crate::datalink::u64_view::U64View;

use super::message::{DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct TrackInfo {
	pub track_id: u16,
	pub contact_id: u32,
	pub contact_type: RadarTargetType,
	pub is_allied: bool,
}

impl TrackInfo {
	pub fn new(track_id: u16, contact_id: u32, contact_type: RadarTargetType, is_allied: bool) -> TrackInfo {
		TrackInfo { track_id, contact_id, contact_type, is_allied }
	}
}

fn radar_to_i32(r: RadarTargetType) -> i32 {
	match r {
		RadarTargetType::SpaceBattleShip => 0,
		RadarTargetType::SpaceHulk => 1,
		RadarTargetType::Missile => 2,
		RadarTargetType::Asteroid => 5,
		RadarTargetType::FlakShell => 7,
		RadarTargetType::APShell => 8,
		_ => -1,
	}
}

fn i32_to_radar(r: i32) -> RadarTargetType {
	match r {
		0 => RadarTargetType::SpaceBattleShip,
		1 => RadarTargetType::SpaceHulk,
		2 => RadarTargetType::Missile,
		5 => RadarTargetType::Asteroid,
		7 => RadarTargetType::FlakShell,
		8 => RadarTargetType::APShell,
		_ => RadarTargetType::Invalid,
	}
}

impl DatalinkMessage for TrackInfo {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.track_id as u64, 12); // 16
		view.write(self.contact_id as u64, 32); // 48
		view.write(radar_to_i32(self.contact_type) as u64, 4); // 52
		view.write(if self.is_allied { 1 } else { 0 }, 1);

		view
	}

	fn parse(mut view: U64View) -> Self {
		let track_id = view.read(12) as u16;
		let contact_id = view.read(32) as u32;
		let contact_type = i32_to_radar(view.read(4) as i32);
		let is_allied = view.read(1) == 1;

		TrackInfo::new(track_id, contact_id, contact_type, is_allied)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::TrackInfo
	}
}
