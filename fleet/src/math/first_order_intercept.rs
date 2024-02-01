use super::vector3::Vector3;

pub fn first_order_intercept(
	shooter_position: Vector3,
	shooter_velocity: Vector3,
	shot_speed: f32,
	target_position: Vector3,
	target_velocity: Vector3,
) -> Vector3 {
	let target_relative_velocity = target_velocity - shooter_velocity;
	let t = first_order_intercept_time(shot_speed, target_position - shooter_position, target_relative_velocity);

	return target_position + target_relative_velocity * t;
}

pub fn first_order_intercept_time(shot_speed: f32, target_relative_position: Vector3, target_relative_velocity: Vector3) -> f32 {
	let velocity_squared = target_relative_velocity.length_sq();
	if velocity_squared < 0.001 {
		return 0.0;
	}

	let a = velocity_squared - shot_speed * shot_speed;
	if a.abs() < 0.001 {
		let t = -target_relative_position.length_sq() / (2.0 * target_relative_velocity.dot(&target_relative_position));
		return f32::max(t, 0.0);
	}

	let b = 2.0 * target_relative_velocity.dot(&target_relative_position);
	let c = target_relative_position.length_sq();
	let detriment = b * b - 4.0 * a * c;

	if detriment > 0.0 {
		let t1 = (-b + detriment.sqrt()) / (2.0 * a);
		let t2 = (-b - detriment.sqrt()) / (2.0 * a);

		if t1 > 0.0 {
			return if t2 > 0.0 { f32::min(t1, t2) } else { t1 };
		} else {
			return t2.max(0.0);
		}
	} else if detriment < 0.0 {
		return 0.0;
	} else {
		return (-b / (2.0 * a)).max(0.0);
	}
}
