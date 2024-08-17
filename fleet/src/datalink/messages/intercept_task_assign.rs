use crate::datalink::u64_view::U64View;

use super::message::{DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct InterceptTaskAssign {
	pub target_id: u16,
	pub contact_id: u32,
	pub interceptor_id: u8,
	pub ring: u8,
}

impl InterceptTaskAssign {
	pub fn new(target_id: u16, contact_id: u32, interceptor_id: u8, ring: u8) -> InterceptTaskAssign {
		InterceptTaskAssign { target_id, contact_id, interceptor_id, ring }
	}
}

impl DatalinkMessage for InterceptTaskAssign {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.target_id as u64, 16); // 20
		view.write(self.contact_id as u64, 32); // 52
		view.write(self.interceptor_id as u64, 8); // 60
		view.write(self.ring as u64, 4); // 64

		view
	}

	fn parse(mut view: U64View) -> Self {
		let target_id = view.read(16) as u16;
		let contact_id = view.read(32) as u32;
		let interceptor_id = view.read(8) as u8;
		let ring = view.read(4) as u8;

		InterceptTaskAssign::new(target_id, contact_id, interceptor_id, ring)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::InterceptTaskAssign
	}
}
