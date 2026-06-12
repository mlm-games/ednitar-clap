pub mod amp;
pub mod cab;
pub mod chain;
pub mod filters;
pub mod fx;
pub mod gate;

pub use chain::GuitarChain;
pub use fx::StereoFx;

/// Lightweight soft clip to prevent nasty overs.
/// Padé approximant of tanh, with input clamped to [-5, 5] to avoid unbounded behavior,
/// and output clamped to [-1, 1] for safety.
#[inline]
pub fn fast_tanh(x: f32) -> f32 {
    let x = x.clamp(-5.0, 5.0);
    let x2 = x * x;
    let y = x * (27.0 + x2) / (27.0 + 9.0 * x2);
    y.clamp(-1.0, 1.0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::db_to_gain;

    #[test]
    fn db_to_gain_known_values() {
        assert!((db_to_gain(0.0) - 1.0).abs() < 1e-6);
        assert!((db_to_gain(-6.0) - 0.501187).abs() < 1e-5);
        assert!((db_to_gain(6.0) - 1.995262).abs() < 1e-5);
        assert!((db_to_gain(-60.0) - 0.001).abs() < 1e-5);
    }

    #[test]
    fn fast_tanh_bounded() {
        for x in [-100.0, -10.0, -5.0, -3.0, 0.0, 3.0, 5.0, 10.0, 100.0] {
            let y = fast_tanh(x);
            assert!(y >= -1.0 && y <= 1.0, "fast_tanh({}) = {} out of bounds", x, y);
        }
    }

    #[test]
    fn fast_tanh_odd_symmetry() {
        let x = 2.0;
        assert!((fast_tanh(x) + fast_tanh(-x)).abs() < 1e-6);
    }

    #[test]
    fn time_to_coeff_sr_independence() {
        let t = 0.01;
        let c1 = time_to_coeff(t, 44100.0);
        let c2 = time_to_coeff(t, 48000.0);
        // both yield pole coeffs for same time constant, should differ slightly
        assert!(c1 > 0.0 && c1 < 1.0);
        assert!(c2 > 0.0 && c2 < 1.0);
        assert!((c1 - c2).abs() > 0.0); // different sample rates give different coeffs
    }

    #[test]
    fn time_to_coeff_edge_cases() {
        assert_eq!(time_to_coeff(0.0, 44100.0), 0.0);
        assert_eq!(time_to_coeff(-1.0, 44100.0), 0.0);
        let c = time_to_coeff(1.0, 44100.0);
        assert!((c - (-1.0 / 44100.0f32).exp()).abs() < 1e-6);
    }

    #[test]
    fn flush_denormals_small_values() {
        assert_eq!(flush_denormals(1e-30), 0.0);
        assert_eq!(flush_denormals(1e-24), 1e-24); // just above threshold
        assert_eq!(flush_denormals(0.5), 0.5);
    }
}
