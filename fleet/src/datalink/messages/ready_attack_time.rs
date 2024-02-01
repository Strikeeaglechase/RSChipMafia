use crate::datalink::u64_view::U64View;

use super::message::{DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct ReadyAttackTime {
	pub time: u32,
}

impl ReadyAttackTime {
	pub fn new(time: u32) -> ReadyAttackTime {
		ReadyAttackTime { time }
	}
}

impl DatalinkMessage for ReadyAttackTime {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.time as u64, 32); // 36

		view
	}

	fn parse(mut view: U64View) -> Self {
		let time = view.read(32) as u32;

		ReadyAttackTime::new(time)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::ReadyAttackTime
	}
}
