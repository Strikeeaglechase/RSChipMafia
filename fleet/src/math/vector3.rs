use rand;

use super::utils::deg;

pub struct Angles {
	pub bearing: f32,
	pub elevation: f32,
}

#[derive(Copy)]
pub struct Vector3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Vector3 {
	pub fn zero() -> Vector3 {
		Vector3 { x: 0.0, y: 0.0, z: 0.0 }
	}

	pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
		Vector3 { x, y, z }
	}

	pub fn random() -> Vector3 {
		Vector3 {
			x: rand::random::<f32>() * 2.0 - 1.0,
			y: rand::random::<f32>() * 2.0 - 1.0,
			z: rand::random::<f32>() * 2.0 - 1.0,
		}
	}

	pub fn random_direction() -> Vector3 {
		let u = (rand::random::<f32>() - 0.5) * 2f32;
		let t = rand::random::<f32>() * std::f32::consts::PI * 2f32;
		let f = (1f32 - u * u).sqrt();
		Vector3 { x: f * t.cos(), y: f * t.sin(), z: u }
	}

	pub fn set(&mut self, x: f32, y: f32, z: f32) {
		self.x = x;
		self.y = y;
		self.z = z;
	}

	// pub fn clone(&self) -> Vector3 {
	// 	Vector3 { x: self.x, y: self.y, z: self.z }
	// }

	pub fn dot(&self, other: &Vector3) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	pub fn cross(&self, other: &Vector3) -> Vector3 {
		Vector3 {
			x: self.y * other.z - self.z * other.y,
			y: self.z * other.x - self.x * other.z,
			z: self.x * other.y - self.y * other.x,
		}
	}

	pub fn length(&self) -> f32 {
		(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
	}

	pub fn length_sq(&self) -> f32 {
		self.x * self.x + self.y * self.y + self.z * self.z
	}

	pub fn normalized(&self) -> Vector3 {
		let len = self.length();
		Vector3 {
			x: self.x / len,
			y: self.y / len,
			z: self.z / len,
		}
	}

	pub fn clamp(&self, min: f32, max: f32) -> Vector3 {
		Vector3 {
			x: self.x.min(max).max(min),
			y: self.y.min(max).max(min),
			z: self.z.min(max).max(min),
		}
	}

	pub fn lerp(&self, other: &Vector3, t: f32) -> Vector3 {
		Vector3 {
			x: self.x + (other.x - self.x) * t,
			y: self.y + (other.y - self.y) * t,
			z: self.z + (other.z - self.z) * t,
		}
	}

	pub fn slerp(&self, other: Vector3, t: f32) -> Vector3 {
		let dot = self.dot(&other).clamp(-1.0, 1.0);
		let theta = dot.acos() * t;
		let relative = other - *self * dot;
		*self * theta.cos() + relative * theta.sin()
	}

	pub fn angles(&self) -> Angles {
		let mut bearing = deg(self.z.atan2(self.x)) + 180.0;
		let mut elevation = -deg((-self.y / (self.x * self.x + self.z * self.z).sqrt()).atan());
		if bearing < 0.0 {
			bearing += 360.0;
		}
		bearing %= 360.0;

		elevation = elevation.clamp(-90.0, 90.0);

		Angles { bearing, elevation }
	}
}

impl Clone for Vector3 {
	fn clone(&self) -> Vector3 {
		Vector3 { x: self.x, y: self.y, z: self.z }
	}
}

impl std::ops::Add for Vector3 {
	type Output = Vector3;

	fn add(self, other: Vector3) -> Vector3 {
		Vector3 {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z,
		}
	}
}

impl std::ops::Sub for Vector3 {
	type Output = Vector3;

	fn sub(self, other: Vector3) -> Vector3 {
		Vector3 {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z,
		}
	}
}

impl std::ops::Mul<f32> for Vector3 {
	type Output = Vector3;

	fn mul(self, other: f32) -> Vector3 {
		Vector3 {
			x: self.x * other,
			y: self.y * other,
			z: self.z * other,
		}
	}
}

impl std::ops::Div<f32> for Vector3 {
	type Output = Vector3;

	fn div(self, other: f32) -> Vector3 {
		Vector3 {
			x: self.x / other,
			y: self.y / other,
			z: self.z / other,
		}
	}
}

impl std::ops::Neg for Vector3 {
	type Output = Vector3;

	fn neg(self) -> Vector3 {
		Vector3 { x: -self.x, y: -self.y, z: -self.z }
	}
}

impl std::ops::AddAssign for Vector3 {
	fn add_assign(&mut self, other: Vector3) {
		self.x += other.x;
		self.y += other.y;
		self.z += other.z;
	}
}

impl std::ops::SubAssign for Vector3 {
	fn sub_assign(&mut self, other: Vector3) {
		self.x -= other.x;
		self.y -= other.y;
		self.z -= other.z;
	}
}

impl std::ops::MulAssign<f32> for Vector3 {
	fn mul_assign(&mut self, other: f32) {
		self.x *= other;
		self.y *= other;
		self.z *= other;
	}
}

impl std::ops::DivAssign<f32> for Vector3 {
	fn div_assign(&mut self, other: f32) {
		self.x /= other;
		self.y /= other;
		self.z /= other;
	}
}

impl std::fmt::Display for Vector3 {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({}, {}, {})", self.x, self.y, self.z)
	}
}

impl std::fmt::Debug for Vector3 {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({}, {}, {})", self.x, self.y, self.z)
	}
}

impl From<(f32, f32, f32)> for Vector3 {
	fn from(t: (f32, f32, f32)) -> Vector3 {
		Vector3 { x: t.0, y: t.1, z: t.2 }
	}
}
