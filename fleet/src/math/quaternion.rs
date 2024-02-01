use super::vector3::Vector3;

pub struct AxisAngle {
	pub axis: Vector3,
	pub angle: f32,
}

pub struct Quaternion {
	pub w: f32,
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Quaternion {
	pub fn zero() -> Quaternion {
		Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
	}

	pub fn new(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
		Quaternion { x, y, z, w }
	}

	pub fn from_euler(euler: &Vector3) -> Quaternion {
		let c1 = (euler.x / 2.0).cos();
		let c2 = (euler.y / 2.0).cos();
		let c3 = (euler.z / 2.0).cos();
		let s1 = (euler.x / 2.0).sin();
		let s2 = (euler.y / 2.0).sin();
		let s3 = (euler.z / 2.0).sin();

		Quaternion {
			w: s1 * c2 * c3 + c1 * s2 * s3,
			x: c1 * s2 * c3 - s1 * c2 * s3,
			y: c1 * c2 * s3 + s1 * s2 * c3,
			z: c1 * c2 * c3 - s1 * s2 * s3,
		}
	}

	pub fn from_vectors(v_from: &Vector3, v_to: &Vector3) -> Quaternion {
		let mut r = v_from.dot(v_to) + 1.0;

		if r < f32::EPSILON {
			r = 0.0;

			if f32::abs(v_from.x) > f32::abs(v_from.z) {
				return Quaternion { x: -v_from.y, y: v_from.x, z: 0.0, w: r };
			} else {
				return Quaternion { x: 0.0, y: -v_from.z, z: v_from.y, w: r };
			}
		}

		return Quaternion {
			x: v_from.y * v_to.z - v_from.z * v_to.y,
			y: v_from.z * v_to.x - v_from.x * v_to.z,
			z: v_from.x * v_to.y - v_from.y * v_to.x,
			w: r,
		}
		.normalized();
	}

	pub fn from_axis_angle(axis_angle: &AxisAngle) -> Quaternion {
		let half_angle = axis_angle.angle / 2.0;
		let s = half_angle.sin();

		Quaternion {
			x: axis_angle.axis.x * s,
			y: axis_angle.axis.y * s,
			z: axis_angle.axis.z * s,
			w: half_angle.cos(),
		}
	}

	pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) {
		self.x = x;
		self.y = y;
		self.z = z;
		self.w = w;
	}

	pub fn angle_to(&self, other: &Quaternion) -> f32 {
		let dot = self.dot(other);
		return f32::acos(f32::min(f32::abs(dot), 1.0)) * 2.0;
	}

	pub fn invert(&self) -> Quaternion {
		Quaternion { x: -self.x, y: -self.y, z: -self.z, w: self.w }
	}

	pub fn dot(&self, other: &Quaternion) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
	}

	pub fn length(&self) -> f32 {
		(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
	}

	pub fn length_sq(&self) -> f32 {
		self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
	}

	pub fn normalized(&self) -> Quaternion {
		let len = self.length();
		if len == 0.0 {
			return Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
		}

		Quaternion {
			x: self.x / len,
			y: self.y / len,
			z: self.z / len,
			w: self.w / len,
		}
	}

	pub fn axis_angle(&self) -> AxisAngle {
		let quat = if self.w > 1.0 { self.normalized() } else { *self };

		let angle = 2.0 * quat.w.acos();
		let den = (1.0 - quat.w * quat.w).sqrt();

		return if den > f32::EPSILON {
			AxisAngle {
				axis: Vector3 {
					x: quat.x / den,
					y: quat.y / den,
					z: quat.z / den,
				},
				angle,
			}
		} else {
			AxisAngle { axis: Vector3 { x: 1.0, y: 0.0, z: 0.0 }, angle }
		};
	}
}

impl Clone for Quaternion {
	fn clone(&self) -> Quaternion {
		Quaternion { x: self.x, y: self.y, z: self.z, w: self.w }
	}
}

impl std::ops::Mul for Quaternion {
	type Output = Quaternion;

	fn mul(self, other: Quaternion) -> Quaternion {
		Quaternion {
			w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
			x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
			y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
			z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
		}
	}
}

impl std::ops::Mul<Vector3> for Quaternion {
	type Output = Vector3;

	fn mul(self, other: Vector3) -> Vector3 {
		let ix = self.w * other.x + self.y * other.z - self.z * other.y;
		let iy = self.w * other.y - self.x * other.z + self.z * other.x;
		let iz = self.w * other.z + self.x * other.y - self.y * other.x;
		let iw = -self.x * other.x - self.y * other.y - self.z * other.z;

		Vector3 {
			x: ix * self.w + iw * -self.x + iy * -self.z - iz * -self.y,
			y: iy * self.w + iw * -self.y + iz * -self.x - ix * -self.z,
			z: iz * self.w + iw * -self.z + ix * -self.y - iy * -self.x,
		}
	}
}

impl Copy for Quaternion {}

impl std::fmt::Display for Quaternion {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
	}
}

impl std::fmt::Debug for Quaternion {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
	}
}

impl From<(f32, f32, f32, f32)> for Quaternion {
	fn from(t: (f32, f32, f32, f32)) -> Quaternion {
		Quaternion { x: t.0, y: t.1, z: t.2, w: t.3 }
	}
}
