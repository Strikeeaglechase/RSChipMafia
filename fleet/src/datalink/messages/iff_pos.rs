use crate::{datalink::u64_view::U64View, math::vector3::Vector3};

use super::message::{squash_f32, unsquash_f32, DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct IFFPosition {
	pub position: Vector3,
	pub dl_id: u8,
}

impl IFFPosition {
	pub fn new(position: Vector3, dl_id: u8) -> IFFPosition {
		IFFPosition { position, dl_id }
	}
}

impl DatalinkMessage for IFFPosition {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		let x = squash_f32(self.position.x, 16, 3, 10000.0);
		let y = squash_f32(self.position.y, 16, 3, 10000.0);
		let z = squash_f32(self.position.z, 16, 3, 10000.0);

		view.write(self.message_type() as u64, 4); // 4
		view.write(x, 16); // 20
		view.write(y, 16); // 36
		view.write(z, 16); // 52
		view.write(self.dl_id as u64, 8); // 60

		return view;
	}

	fn parse(mut view: U64View) -> Self {
		let x = view.read(16) as u64;
		let y = view.read(16) as u64;
		let z = view.read(16) as u64;
		let dl_id = view.read(8) as u8;

		let position = Vector3::new(
			unsquash_f32(x, 16, 3, 10000.0),
			unsquash_f32(y, 16, 3, 10000.0),
			unsquash_f32(z, 16, 3, 10000.0),
		);

		return IFFPosition::new(position, dl_id);
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::IFFPosition
	}
}
