use crate::{datalink::u64_view::U64View, math::vector3::Vector3};

use super::message::{squash_f32, unsquash_f32, DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct TrackPosition {
	pub track_id: u16,
	pub position: Vector3,
}

impl TrackPosition {
	pub fn new(track_id: u16, position: Vector3) -> TrackPosition {
		TrackPosition { track_id, position }
	}
}

impl DatalinkMessage for TrackPosition {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		let x = squash_f32(self.position.x, 16, 3, 10000.0);
		let y = squash_f32(self.position.y, 16, 3, 10000.0);
		let z = squash_f32(self.position.z, 16, 3, 10000.0);

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.track_id as u64, 12); // 16
		view.write(x, 16); // 32
		view.write(y, 16); // 48
		view.write(z, 16); // 64

		view
	}

	fn parse(mut view: U64View) -> Self {
		let track_id = view.read(12) as u16;
		let x = view.read(16) as u64;
		let y = view.read(16) as u64;
		let z = view.read(16) as u64;

		let position = Vector3::new(
			unsquash_f32(x, 16, 3, 10000.0),
			unsquash_f32(y, 16, 3, 10000.0),
			unsquash_f32(z, 16, 3, 10000.0),
		);

		TrackPosition::new(track_id, position)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::TrackPosition
	}
}
