use crate::{datalink::u64_view::U64View, math::vector3::Vector3};

use super::message::{squash_f32, unsquash_f32, DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct TrackVelocity {
	pub track_id: u16,
	pub velocity: Vector3,
}

impl TrackVelocity {
	pub fn new(track_id: u16, velocity: Vector3) -> TrackVelocity {
		TrackVelocity { track_id, velocity }
	}
}

impl DatalinkMessage for TrackVelocity {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		let x = squash_f32(self.velocity.x, 16, 45, 750.0);
		let y = squash_f32(self.velocity.y, 16, 45, 750.0);
		let z = squash_f32(self.velocity.z, 16, 45, 750.0);

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

		let velocity = Vector3::new(unsquash_f32(x, 16, 45, 750.0), unsquash_f32(y, 16, 45, 750.0), unsquash_f32(z, 16, 45, 750.0));

		TrackVelocity::new(track_id, velocity)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::TrackVelocity
	}
}
