use std::{cell::RefCell, collections::HashMap};

use protologic_core::{
	radar::RadarTargetType,
	radio::{radio_receive, radio_receive_filter, radio_transmit},
};
use rand::random;

use crate::{
	datalink::messages::{join_request::JoinRequest, message::DatalinkMessage},
	get,
	math::{utils::now, vector3::Vector3},
};

use super::messages::{
	leave_network::LeaveNetwork,
	message::Message,
	net_info::{self, NetInfo},
	track_id::TrackId,
};

#[derive(Clone, Copy)]
pub struct DatalinkTrack {
	pub track_id: u16,
	pub contact_id: u32,
	pub contact_type: RadarTargetType,

	pub position: Vector3,
	pub velocity: Vector3,
	pub last_update_timestamp: f32,
}

impl DatalinkTrack {
	pub fn new(track_id: u16) -> DatalinkTrack {
		DatalinkTrack {
			track_id,
			contact_id: 0,
			contact_type: RadarTargetType::Invalid,

			position: Vector3::zero(),
			velocity: Vector3::zero(),

			last_update_timestamp: f32::MAX,
		}
	}

	pub fn update(&mut self, dt: f32) {
		self.position += self.velocity * dt;
	}

	pub fn update_position(&mut self, position: Vector3) {
		self.position = position;
		self.last_update_timestamp = now();
	}

	pub fn update_velocity(&mut self, velocity: Vector3) {
		self.velocity = velocity;
		self.last_update_timestamp = now();
	}
}

#[derive(PartialEq)]
pub enum DatalinkStatus {
	None,
	WaitingForId,
	Joined,
	Disconnected,
}

struct TimeBlock {
	index: u8,
	clients: Vec<u8>,
}

impl TimeBlock {
	pub fn new(index: u8) -> TimeBlock {
		TimeBlock { index, clients: Vec::new() }
	}
}

pub struct Datalink {
	status: DatalinkStatus,
	blocks: Vec<TimeBlock>,

	join_request_approve_id: u8,
	our_join_request_id: u8,
	our_request_block: u8,

	id: u8,

	total_blocks: u8,
	our_block: u8,

	next_free_block: u8,

	message_queue: Vec<Message>,
	core_message_queue: Vec<Message>,
	disconnect_on_next_send: bool,

	id_map: HashMap<u32, u16>,
	next_track_id: u16,
	next_id: u8,
	tracks: Vec<DatalinkTrack>,

	is_host: bool,
	pub tick: u32,
}

const INVALID: u8 = u8::MAX;

impl Datalink {
	fn new() -> Datalink {
		Datalink {
			status: DatalinkStatus::None,
			blocks: Vec::new(),

			join_request_approve_id: INVALID,
			our_join_request_id: INVALID,
			our_request_block: INVALID,

			id: INVALID,

			total_blocks: 0,
			our_block: 0,
			next_free_block: INVALID,

			message_queue: Vec::new(),
			core_message_queue: Vec::new(),
			disconnect_on_next_send: false,

			id_map: HashMap::new(),
			next_track_id: 0,
			next_id: 0,
			tracks: Vec::new(),

			is_host: false,
			tick: 0,
		}
	}

	fn setup_as_host(&mut self) {
		self.id = 0;
		self.status = DatalinkStatus::Joined;

		let block0 = TimeBlock { index: 0, clients: vec![0] }; // Networking management, and host slot
		self.blocks.push(block0);

		let block1 = TimeBlock { index: 1, clients: vec![0, 0] }; // New joiner slot
		self.blocks.push(block1);

		self.total_blocks = 2;
		self.our_block = 1;

		self.is_host = true;
	}

	fn is_our_turn(&self) -> bool {
		u32::from(self.our_block) == self.tick % u32::from(self.total_blocks)
	}

	fn is_host_transmit_turn(&self) -> bool {
		0 == (self.tick + 1) % u32::from(self.total_blocks) && self.is_host
	}

	fn update(&mut self) {
		if self.status == DatalinkStatus::Disconnected {
			return;
		}

		let mut buffer: Vec<u64> = Vec::new();
		radio_receive_filter(0, 0);
		radio_receive(&mut buffer);

		for message in buffer {
			self.handle_packet(message);
		}

		if self.status != DatalinkStatus::Joined {
			return;
		}

		if self.is_host_transmit_turn() {
			self.send_net_info();
		}

		if self.is_our_turn() {
			if self.disconnect_on_next_send {
				// Send disconnect message
				let leave_network = LeaveNetwork::new(self.our_block, self.id);
				self.transmit(leave_network.serialize().value);
				self.status = DatalinkStatus::Disconnected;
				println!("Datalink {} disconnected", self.id);
				return;
			}

			if self.message_queue.len() > 0 {
				let next_message = self.message_queue.remove(0);
				self.transmit(next_message.serialize().value);
			}

			if self.message_queue.len() > 15 {
				println!("Datalink {} has excessive message queue size: {}", self.id, self.message_queue.len());
				for message in &self.message_queue {
					println!("{:?}", message);
				}
			}
		}

		self.tick += 1;
	}

	fn get_core_message_queue(&mut self) -> Vec<Message> {
		let mut queue: Vec<Message> = Vec::new();

		for message in &self.core_message_queue {
			queue.push(message.clone());
		}

		self.core_message_queue.clear();

		queue
	}

	fn send_message(&mut self, message: Message) {
		self.message_queue.push(message);
	}

	fn transmit(&self, value: u64) {
		radio_transmit(value, f32::MAX);
		// println!("Sending: {:?}", value);
	}

	fn send_net_info(&mut self) {
		let mut next_free_block = INVALID;

		for i in 0..self.blocks.len() {
			let block = &self.blocks[i];
			if block.clients.len() < 4 {
				next_free_block = block.index;
				break;
			}
		}
		if next_free_block == INVALID {
			next_free_block = self.blocks.len() as u8;
		}

		let net_info = NetInfo::new(self.next_id, self.total_blocks, next_free_block, self.tick, self.join_request_approve_id);
		self.transmit(net_info.serialize().value);

		// println!("Sent net info: {:?}", net_info);
		self.join_request_approve_id = INVALID;
		self.total_blocks = self.blocks.len() as u8;
	}

	fn handle_packet(&mut self, message: u64) {
		let packet = Message::parse(message);

		match packet {
			Message::NetInfo(net_info) => self.handle_net_info(net_info),
			Message::JoinRequest(join_request) => self.process_join_request(join_request),
			Message::TrackId(track_id) => self.process_track_id(track_id),
			Message::LeaveRequest(leave_network) => self.process_leave_network(leave_network),
			Message::TrackInfo(track_info) => {
				let track = self.get_track_mut(track_info.track_id);
				track.contact_id = track_info.contact_id;
				track.contact_type = track_info.contact_type;
			}
			Message::TrackPosition(track_position) => self.get_track_mut(track_position.track_id).update_position(track_position.position),
			Message::TrackVelocity(track_velocity) => self.get_track_mut(track_velocity.track_id).update_velocity(track_velocity.velocity),
			_ => self.core_message_queue.push(packet),
		}
	}

	fn handle_net_info(&mut self, net_info: net_info::NetInfo) {
		self.total_blocks = net_info.num_blocks;
		self.tick = net_info.current_tick + 1;
		self.next_free_block = net_info.next_free_block;

		match self.status {
			DatalinkStatus::WaitingForId => {
				if net_info.join_request_approve_id == self.our_join_request_id {
					// Accepted into the network
					self.status = DatalinkStatus::Joined;
					self.id = net_info.next_id;
					self.our_block = self.our_request_block;
					println!("Joined network with id {}", self.id);
				} else {
					// Denied
					self.send_join_request();
				}
			}
			DatalinkStatus::None => {
				self.send_join_request();
			}
			_ => {}
		}
	}

	fn send_join_request(&mut self) {
		let request_id = random::<u8>();
		println!("Sending join request with id {}", request_id);

		let join_request_packet = JoinRequest::new(request_id, self.next_free_block);
		self.transmit(join_request_packet.serialize().value);

		self.our_join_request_id = request_id;
		self.our_request_block = self.next_free_block;

		self.status = DatalinkStatus::WaitingForId;
	}

	fn get_block(&mut self, block_index: u8) -> Option<&mut TimeBlock> {
		self.blocks.iter_mut().find(|block| block.index == block_index)
	}

	fn process_track_id(&mut self, packet: TrackId) {
		if packet.track_id > 0 {
			self.id_map.insert(packet.contact_id, packet.track_id);
		}

		// Someone is requesting a track id
		if self.is_host && packet.track_id == 0 {
			let new_id = self.next_track_id;
			self.next_track_id += 1;
			self.id_map.insert(packet.contact_id, new_id);

			let reply = TrackId::new(new_id, packet.contact_id);
			self.message_queue.push(Message::TrackId(reply));
		}
	}

	fn process_join_request(&mut self, join_request: JoinRequest) {
		if !self.is_host {
			return;
		}

		if self.join_request_approve_id != INVALID {
			println!(
				"Failed to approve join request {} because {} is already waiting for approval",
				join_request.request_id, self.join_request_approve_id
			);
			return;
		}

		let next_id = self.next_id + 1;
		let request_block = self.get_block(join_request.block);
		match request_block {
			Some(block) => {
				if block.clients.len() < 4 {
					block.clients.push(next_id);
					self.accept_join_request(join_request.request_id);
				}
			}
			None => {
				let mut new_block = TimeBlock::new(join_request.block);
				new_block.clients.push(next_id);
				self.blocks.push(new_block);
				self.accept_join_request(join_request.request_id);
			}
		}
	}

	fn accept_join_request(&mut self, request_id: u8) {
		self.join_request_approve_id = request_id;
		self.next_id += 1;

		println!("Accepted join request {}", request_id);
	}

	fn process_leave_network(&mut self, leave_network: LeaveNetwork) {
		if !self.is_host {
			return;
		}

		self.remove_client_from_block(leave_network.block, leave_network.id);
	}

	fn remove_client_from_block(&mut self, block: u8, id: u8) {
		let block = get!(self.get_block(block));

		let index = get!(block.clients.iter().position(|f| *f == id));
		block.clients.remove(index);
	}

	fn create_track(&mut self, track_id: u16) -> &mut DatalinkTrack {
		let track = DatalinkTrack::new(track_id);
		self.tracks.push(track);

		self.tracks.last_mut().unwrap()
	}

	fn get_track_mut(&mut self, track_id: u16) -> &mut DatalinkTrack {
		if self.does_track_exist(track_id) {
			self.tracks.iter_mut().find(|f| f.track_id == track_id).unwrap()
		} else {
			self.create_track(track_id)
		}
	}

	fn does_track_exist(&self, track_id: u16) -> bool {
		self.tracks.iter().any(|f| f.track_id == track_id)
	}

	fn get_track(&self, track_id: u16) -> Option<&DatalinkTrack> {
		self.tracks.iter().find(|f| f.track_id == track_id)
	}

	fn update_track_from_local_data(&mut self, contact_id_64: i64, rc_type: RadarTargetType, position: Vector3, velocity: Vector3) {
		let track_id = get!(self.net_id(contact_id_64));

		let track = self.get_track_mut(track_id);
		track.contact_id = dl_crunch_id(contact_id_64);
		track.contact_type = rc_type;
		track.position = position;
		track.velocity = velocity;
		track.last_update_timestamp = now();
	}

	fn get_ship_tracks(&self) -> Vec<DatalinkTrack> {
		self
			.tracks
			.iter()
			.filter(|f| f.contact_type == RadarTargetType::SpaceBattleShip)
			.cloned()
			.collect()
	}

	fn disconnect(&mut self) {
		self.disconnect_on_next_send = true;
	}

	fn net_id(&mut self, contact_id_64: i64) -> Option<u16> {
		let contact_id = dl_crunch_id(contact_id_64);
		if let Some(id) = self.id_map.get(&contact_id) {
			return Some(*id);
		}

		if self.is_host {
			let new_id = self.next_track_id;
			self.next_track_id += 1;
			self.id_map.insert(contact_id, new_id);
			return Some(new_id);
		}

		let mut has_pending_request = false;
		for message in &self.message_queue {
			if let Message::TrackId(track_id) = message {
				if track_id.contact_id == contact_id {
					has_pending_request = true;
					break;
				}
			}
		}

		if !has_pending_request {
			let track_id = TrackId::new(0, contact_id);
			self.message_queue.push(Message::TrackId(track_id));
		}

		None
	}
}

thread_local! {
	static FC: RefCell<Datalink> = RefCell::new(Datalink::new());
}

pub fn dl_declare_host() {
	FC.with(|fc| {
		let mut fc = fc.borrow_mut();
		fc.setup_as_host();
	});
}

pub fn update_dl() {
	FC.with(|fc| {
		let mut fc = fc.borrow_mut();
		fc.update();
	});
}

pub fn get_tick() -> u32 {
	FC.with(|fc| {
		let fc = fc.borrow();
		fc.tick
	})
}

pub fn send_message(message: Message) {
	FC.with(|fc| {
		let mut fc = fc.borrow_mut();
		fc.send_message(message);
	});
}

pub fn get_core_message_queue() -> Vec<Message> {
	FC.with(|fc| {
		let mut fc = fc.borrow_mut();
		fc.get_core_message_queue()
	})
}

pub fn get_dl_track(track_id: u16) -> Option<DatalinkTrack> {
	FC.with(|fc| fc.borrow().get_track(track_id).copied())
}

pub fn get_ship_tracks() -> Vec<DatalinkTrack> {
	FC.with(|fc| fc.borrow().get_ship_tracks())
}

pub fn update_track_from_local_data(contact_id_64: i64, rc_type: RadarTargetType, position: Vector3, velocity: Vector3) {
	FC.with(|fc| fc.borrow_mut().update_track_from_local_data(contact_id_64, rc_type, position, velocity));
}

pub fn datalink_disconnect() {
	FC.with(|fc| {
		fc.borrow_mut().disconnect();
	});
}

pub fn dl_net_id(contact_id: i64) -> Option<u16> {
	FC.with(|fc| fc.borrow_mut().net_id(contact_id))
}

pub fn dl_crunch_id(contact_id: i64) -> u32 {
	(contact_id >> 32) as u32
}

pub fn own_dl_id() -> u8 {
	FC.with(|fc| fc.borrow().id)
}
