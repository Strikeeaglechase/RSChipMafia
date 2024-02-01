pub struct U64View {
	pub value: u64,

	write_index: usize,
	read_index: usize,
}

impl U64View {
	pub fn zero() -> U64View {
		U64View::new(0)
	}

	pub fn new(value: u64) -> U64View {
		U64View { value, write_index: 0, read_index: 0 }
	}

	pub fn get(&self, start: usize, len: usize) -> u64 {
		let mask = (1 << len) - 1;
		(self.value >> start) & mask
	}

	pub fn set(&mut self, start: usize, len: usize, value: u64) {
		let mask = (1 << len) - 1;
		self.value &= !(mask << start);
		self.value |= (value & mask) << start;
	}

	pub fn write(&mut self, value: u64, len: usize) {
		self.set(self.write_index, len, value);
		self.write_index += len;
		if self.write_index > 64 {
			panic!("U64View write overflowed");
		}
	}

	pub fn read(&mut self, len: usize) -> u64 {
		let value = self.get(self.read_index, len);
		self.read_index += len;

		if self.read_index > 64 {
			panic!("U64View read overflowed");
		}

		value
	}
}
