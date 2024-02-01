use crate::datalink::u64_view::U64View;

use super::message::{DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct NetInfo {
	pub next_id: u8,
	pub num_blocks: u8,
	pub next_free_block: u8,
	pub current_tick: u32,
	pub join_request_approve_id: u8,
}

impl NetInfo {
	pub fn new(next_id: u8, num_blocks: u8, next_free_block: u8, current_tick: u32, join_request_approve_id: u8) -> NetInfo {
		NetInfo {
			next_id,
			num_blocks,
			next_free_block,
			current_tick,
			join_request_approve_id,
		}
	}
}

impl DatalinkMessage for NetInfo {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.next_id as u64, 8); // 12
		view.write(self.num_blocks as u64, 4); // 16
		view.write(self.next_free_block as u64, 4); // 20
		view.write(self.current_tick as u64, 32); // 52
		view.write(self.join_request_approve_id as u64, 8); // 60

		view
	}

	fn parse(mut view: U64View) -> Self {
		let next_id = view.read(8) as u8;
		let num_blocks = view.read(4) as u8;
		let next_free_block = view.read(4) as u8;
		let current_tick = view.read(32) as u32;
		let join_request_approve_id = view.read(8) as u8;

		NetInfo::new(next_id, num_blocks, next_free_block, current_tick, join_request_approve_id)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::NetInfo
	}
}
