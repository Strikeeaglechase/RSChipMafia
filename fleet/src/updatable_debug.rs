use protologic_core::debugging::{debug_sphere_create, DebugShapeHandle};

use crate::math::{utils::debug_line, vector3::Vector3};

pub struct UpdatableDebugLine {
	handle: Option<DebugShapeHandle>,
	a: Vector3,
	b: Vector3,

	cr: f32,
	cg: f32,
	cb: f32,
}

impl UpdatableDebugLine {
	pub fn new() -> UpdatableDebugLine {
		UpdatableDebugLine {
			handle: None,
			a: Vector3::zero(),
			b: Vector3::zero(),

			cr: 1.0,
			cg: 1.0,
			cb: 1.0,
		}
	}

	pub fn set_a(&mut self, a: Vector3) {
		if (a - self.a).length_sq() < 0.01 {
			return;
		}

		self.a = a;
		self.update();
	}

	pub fn set_b(&mut self, b: Vector3) {
		if (b - self.b).length_sq() < 0.01 {
			return;
		}

		self.b = b;
		self.update();
	}

	pub fn set_color(&mut self, cr: f32, cg: f32, cb: f32) {
		if self.cr == cr && self.cg == cg && self.cb == cb {
			return;
		}

		self.cr = cr;
		self.cg = cg;
		self.cb = cb;
		self.update();
	}

	pub fn update(&mut self) {
		self.handle = Some(debug_line(self.a, self.b, self.cr, self.cg, self.cb));
	}
}

pub struct UpdatableSphere {
	handle: Option<DebugShapeHandle>,
	pos: Vector3,
	radius: f32,

	cr: f32,
	cg: f32,
	cb: f32,
}

impl UpdatableSphere {
	pub fn new() -> UpdatableSphere {
		UpdatableSphere {
			handle: None,
			pos: Vector3::zero(),
			radius: 1.0,

			cr: 1.0,
			cg: 1.0,
			cb: 1.0,
		}
	}

	pub fn set_pos(&mut self, pos: Vector3) {
		if (pos - self.pos).length_sq() < 0.01 {
			return;
		}

		self.pos = pos;
		self.update();
	}

	pub fn set_radius(&mut self, radius: f32) {
		if (radius - self.radius).abs() < 0.01 {
			return;
		}

		self.radius = radius;
		self.update();
	}

	pub fn set_color(&mut self, cr: f32, cg: f32, cb: f32) {
		if self.cr == cr && self.cg == cg && self.cb == cb {
			return;
		}

		self.cr = cr;
		self.cg = cg;
		self.cb = cb;
		self.update();
	}

	pub fn update(&mut self) {
		self.handle = Some(debug_sphere_create(self.pos.x, self.pos.y, self.pos.z, self.radius, self.cr, self.cg, self.cb));
	}

	pub fn remove(&mut self) {
		if self.handle.is_some() {
			self.handle = None; // Drop
		}
	}
}
