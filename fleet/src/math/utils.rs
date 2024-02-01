use std::time::SystemTime;

use protologic_core::debugging::{debug_line_create, DebugShapeHandle};

use super::vector3::Vector3;

pub fn deg(radians: f32) -> f32 {
	radians * 180.0 / std::f32::consts::PI
}
pub fn rad(degrees: f32) -> f32 {
	degrees * std::f32::consts::PI / 180.0
}
pub fn now() -> f32 {
	SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f32()
}
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a + (b - a) * t
}

pub fn debug_line_white(a: Vector3, b: Vector3) -> DebugShapeHandle {
	debug_line(a, b, 1.0, 1.0, 1.0)
}

pub fn debug_line(a: Vector3, b: Vector3, cr: f32, cg: f32, cb: f32) -> DebugShapeHandle {
	debug_line_create(a.x, a.y, a.z, b.x, b.y, b.z, cr, cg, cb)
}

#[macro_export]
macro_rules! get {
	($e:expr) => {
		if let Some(x) = $e {
			x
		} else {
			return;
		}
	};
}
