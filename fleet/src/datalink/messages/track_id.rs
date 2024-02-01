use crate::datalink::u64_view::U64View;

use super::message::{DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct TrackId {
	pub track_id: u16,
	pub contact_id: u32,
}

impl TrackId {
	pub fn new(track_id: u16, contact_id: u32) -> TrackId {
		TrackId { track_id, contact_id }
	}
}

impl DatalinkMessage for TrackId {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.track_id as u64, 16); // 20
		view.write(self.contact_id as u64, 32); // 52

		view
	}

	fn parse(mut view: U64View) -> Self {
		let track_id = view.read(16) as u16;
		let contact_id = view.read(32) as u32;

		TrackId::new(track_id, contact_id)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::TrackId
	}
}
