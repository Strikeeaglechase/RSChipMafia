use crate::datalink::u64_view::U64View;

use super::message::{DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct JoinRequest {
	pub request_id: u8,
	pub block: u8,
}

impl JoinRequest {
	pub fn new(request_id: u8, block: u8) -> JoinRequest {
		JoinRequest { request_id, block }
	}
}

impl DatalinkMessage for JoinRequest {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.request_id as u64, 8); // 12
		view.write(self.block as u64, 4); // 16

		view
	}

	fn parse(mut view: U64View) -> Self {
		let request_id = view.read(8) as u8;
		let block = view.read(4) as u8;

		JoinRequest::new(request_id, block)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::JoinRequest
	}
}
