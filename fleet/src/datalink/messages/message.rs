use enum_mac::EnumKeys;

use crate::datalink::u64_view::U64View;

use super::{
	assign_attack_target::AssignAttackTarget, iff_pos::IFFPosition, intercept_task_assign::InterceptTaskAssign, join_request::JoinRequest,
	leave_network::LeaveNetwork, net_info::NetInfo, ready_attack_time::ReadyAttackTime, track_id::TrackId, track_info::TrackInfo, track_position::TrackPosition,
	track_velocity::TrackVelocity,
};

pub trait DatalinkMessage {
	fn serialize(&self) -> U64View;
	fn parse(view: U64View) -> Self;

	fn message_type(&self) -> MessageKey;
}

#[derive(EnumKeys, Clone, Debug)]
pub enum Message {
	NetInfo(NetInfo),
	JoinRequest(JoinRequest),
	LeaveRequest(LeaveNetwork),

	TrackId(TrackId),
	TrackPosition(TrackPosition),
	TrackVelocity(TrackVelocity),

	ReadyAttackTime(ReadyAttackTime),
	TrackInfo(TrackInfo),
	AssignAttackTarget(AssignAttackTarget),

	IFFPosition(IFFPosition),
	InterceptTaskAssign(InterceptTaskAssign),
}

impl Message {
	pub fn parse(message: u64) -> Message {
		let mut view = U64View::new(message);

		let message_type: MessageKey = (view.read(4) as u8).into();

		match message_type {
			MessageKey::NetInfo => Message::NetInfo(NetInfo::parse(view)),
			MessageKey::JoinRequest => Message::JoinRequest(JoinRequest::parse(view)),
			MessageKey::LeaveRequest => Message::LeaveRequest(LeaveNetwork::parse(view)),
			MessageKey::TrackId => Message::TrackId(TrackId::parse(view)),
			MessageKey::TrackPosition => Message::TrackPosition(TrackPosition::parse(view)),
			MessageKey::TrackVelocity => Message::TrackVelocity(TrackVelocity::parse(view)),
			MessageKey::ReadyAttackTime => Message::ReadyAttackTime(ReadyAttackTime::parse(view)),
			MessageKey::TrackInfo => Message::TrackInfo(TrackInfo::parse(view)),
			MessageKey::AssignAttackTarget => Message::AssignAttackTarget(AssignAttackTarget::parse(view)),
			MessageKey::IFFPosition => Message::IFFPosition(IFFPosition::parse(view)),
			MessageKey::InterceptTaskAssign => Message::InterceptTaskAssign(InterceptTaskAssign::parse(view)),
		}
	}

	pub fn serialize(&self) -> U64View {
		return match self {
			Message::NetInfo(net_info) => net_info.serialize(),
			Message::JoinRequest(join_request) => join_request.serialize(),
			Message::LeaveRequest(leave_request) => leave_request.serialize(),
			Message::TrackId(track_id) => track_id.serialize(),
			Message::TrackPosition(track_position) => track_position.serialize(),
			Message::TrackVelocity(track_velocity) => track_velocity.serialize(),
			Message::ReadyAttackTime(ready_attack_time) => ready_attack_time.serialize(),
			Message::TrackInfo(track_info) => track_info.serialize(),
			Message::AssignAttackTarget(assign_attack_target) => assign_attack_target.serialize(),
			Message::IFFPosition(iff_position) => iff_position.serialize(),
			Message::InterceptTaskAssign(intercept_task_assign) => intercept_task_assign.serialize(),
		};
	}
}

pub fn squash_f32(inp_value: f32, bits: usize, ratio: i32, offset: f32) -> u64 {
	let value = ((inp_value + offset) * (ratio as f32)).round() as i32;
	let mask = (1 << bits) - 1;

	value as u64 & mask as u64
}

pub fn unsquash_f32(inp_value: u64, bits: usize, ratio: i32, offset: f32) -> f32 {
	let mask = (1 << bits) - 1;
	let value = inp_value & mask as u64;

	(value as i32 as f32) / (ratio as f32) - offset
}
