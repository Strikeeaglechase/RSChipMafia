use crate::datalink::u64_view::U64View;

use super::message::{DatalinkMessage, MessageKey};

#[derive(Clone, Debug)]
pub struct AssignAttackTarget {
	pub target_id: u16,
}

impl AssignAttackTarget {
	pub fn new(target_id: u16) -> AssignAttackTarget {
		AssignAttackTarget { target_id }
	}
}

impl DatalinkMessage for AssignAttackTarget {
	fn serialize(&self) -> U64View {
		let mut view = U64View::zero();

		view.write(self.message_type() as u64, 4); // 4
		view.write(self.target_id as u64, 16); // 20

		view
	}

	fn parse(mut view: U64View) -> Self {
		let target_id = view.read(16) as u16;

		AssignAttackTarget::new(target_id)
	}

	fn message_type(&self) -> MessageKey {
		MessageKey::AssignAttackTarget
	}
}
