pub struct PID {
	p: f32,
	i: f32,
	d: f32,

	prev_error: f32,
	integral: f32,

	max_integral: f32,
}

impl PID {
	pub fn new(p: f32, i: f32, d: f32, max_integral: f32) -> PID {
		PID {
			p,
			i,
			d,

			prev_error: 0.0,
			integral: 0.0,

			max_integral,
		}
	}

	pub fn update(&mut self, error: f32, dt: f32) -> f32 {
		self.integral += error * dt;
		self.integral = self.integral.min(self.max_integral).max(-self.max_integral);

		let derivative = (error - self.prev_error) / dt;
		self.prev_error = error;

		self.p * error + self.i * self.integral + self.d * derivative
	}
}
