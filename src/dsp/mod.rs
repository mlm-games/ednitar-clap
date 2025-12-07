pub mod amp;
pub mod cab;
pub mod chain;
pub mod filters;
pub mod fx;
pub mod gate;

pub use chain::GuitarChain;
pub use fx::StereoFx;

/// Lightweight soft clip to prevent nasty overs
#[inline]
pub fn fast_tanh(x: f32) -> f32 {
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

#[inline]
pub fn flush_denormals(x: f32) -> f32 {
    if x.abs() < 1e-24 {
        0.0
    } else {
        x
    }
}

/// Helper: coefficient for time-constant based smoothing.
/// `time_s` is seconds to decay ~63%.
#[inline]
pub fn time_to_coeff(time_s: f32, sr: f32) -> f32 {
    if time_s <= 0.0 {
        0.0
    } else {
        (-1.0 / (time_s * sr)).exp()
    }
}
