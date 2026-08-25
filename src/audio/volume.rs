pub const SILENCE_DB: f32 = -80.0;

pub fn linear_to_decibels(linear: f32) -> f32 {
    if linear <= 0.0 {
        SILENCE_DB
    } else {
        (20.0 * linear.log10()).max(SILENCE_DB)
    }
}
