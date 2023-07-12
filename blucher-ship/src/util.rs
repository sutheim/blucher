pub fn smooth_damp(
    current: f32,
    target: f32,
    current_vel: &mut f32,
    time: f32,
    delta_time: f32,
) -> f32 {
    let time = f32::max(0.00001, time);
    let omega = 2f32 / time;

    let x = omega * delta_time;
    let exp = 1f32 / (1f32 + x + 0.48f32 * x * x + 0.235f32 * x * x * x);
    let change = current - target;
    let original_to = target;

    let temp = (*current_vel + omega * change) * delta_time;
    *current_vel = (*current_vel - omega * temp) * exp;

    let mut output = target + (change + temp) * exp;

    if (original_to - current > 0f32) == (output > original_to) {
        output = target + (change + temp) * exp;
        *current_vel = (output - original_to) / delta_time;
    }

    output
}
