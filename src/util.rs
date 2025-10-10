use bevy::math::FloatExt;

#[inline]
pub fn actuator(value: f32, target: f32, down_speed: f32, up_speed: f32) -> f32 {
    if value + up_speed < target {
        value + up_speed
    } else if value + down_speed > target {
        value + down_speed
    } else {
        target
    }
}

#[inline]
pub fn limit(input: f32, lower: f32, upper: f32) -> f32 {
    input.clamp(lower, upper)
}

#[inline]
pub fn rescale(input: f32, min: f32, max: f32) -> f32 {
    if input >= 0.0 {
        input * max.abs()
    } else {
        input * min.abs()
    }
}

#[inline(always)]
pub fn table_lerp(xs: &[f32], ys: &[f32], x: f32) -> f32 {
    let n = xs.len();
    if n == 0 {
        return 0.0;
    }
    if x <= xs[0] {
        return ys[0];
    }
    if x >= xs[n - 1] {
        return ys[n - 1];
    }

    for i in 0..n - 1 {
        if xs[i] <= x && x < xs[i + 1] {
            let t = (x - xs[i]) / (xs[i + 1] - xs[i]);
            return ys[i].lerp(ys[i + 1], t);
        }
    }

    ys[n - 1]
}
