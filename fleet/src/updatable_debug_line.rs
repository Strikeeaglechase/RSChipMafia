use protologic_core::debugging::DebugShapeHandle;

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
