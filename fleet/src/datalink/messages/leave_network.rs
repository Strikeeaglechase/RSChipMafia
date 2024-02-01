use crate::datalink::u64_view::U64View;

use super::message::{DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct LeaveNetwork {
	pub block: u8,
	pub id: u8,
}

impl LeaveNetwork {
	pub fn new(block: u8, id: u8) -> LeaveNetwork {
		LeaveNetwork { block, id }
	}
}

impl DatalinkMessage for LeaveNetwork {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.block as u64, 4); // 8
		view.write(self.id as u64, 8); // 16

		view
	}

	fn parse(mut view: U64View) -> Self {
		let block = view.read(4) as u8;
		let id = view.read(8) as u8;

		LeaveNetwork::new(block, id)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::LeaveRequest
	}
}

impl Copy for LeaveNetwork {}
