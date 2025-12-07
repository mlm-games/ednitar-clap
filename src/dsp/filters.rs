use crate::dsp::flush_denormals;
use core::f32::consts::PI;

/// One-pole low-pass filter.
pub struct OnePoleLp {
    a: f32,
    z: f32,
}

impl OnePoleLp {
    pub fn new() -> Self {
        Self { a: 0.0, z: 0.0 }
    }

    pub fn set_cutoff(&mut self, sr: f32, hz: f32) {
        let hz = hz.max(1.0).min(sr * 0.45);
        let a = (-2.0 * PI * hz / sr).exp();
        self.a = 1.0 - a;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.z += self.a * (x - self.z);
        flush_denormals(self.z)
    }
}

/// One-pole high-pass built from an internal LP: hp = x - lp.
pub struct OnePoleHp {
    lp: OnePoleLp,
}

impl OnePoleHp {
    pub fn new() -> Self {
        Self {
            lp: OnePoleLp::new(),
        }
    }

    pub fn set_cutoff(&mut self, sr: f32, hz: f32) {
        self.lp.set_cutoff(sr, hz);
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let lp = self.lp.process(x);
        flush_denormals(x - lp)
    }
}

/// Simple tilt EQ: splits signal into low/high bands using one LP,
/// then applies opposite gains to low vs high.
///
/// `tilt` in [-1, 1]: -1 = darker (more lows), +1 = brighter (more highs).
pub struct TiltEq {
    lp: OnePoleLp,
    strength: f32,
}

impl TiltEq {
    pub fn new() -> Self {
        Self {
            lp: OnePoleLp::new(),
            strength: 0.8, // how strong the tilt can be
        }
    }

    pub fn set_pivot(&mut self, sr: f32, hz: f32) {
        self.lp.set_cutoff(sr, hz);
    }

    #[inline]
    pub fn process(&mut self, x: f32, tilt: f32) -> f32 {
        let tilt = tilt.clamp(-1.0, 1.0);
        let low = self.lp.process(x);
        let high = x - low;
        let k = tilt * self.strength;

        // low up, high down or vice versa
        let low_gain = 1.0 + (-k);
        let high_gain = 1.0 + k;

        flush_denormals(low * low_gain + high * high_gain)
    }
}
