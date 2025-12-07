use crate::dsp::{flush_denormals, time_to_coeff};
use crate::params::{db_to_gain, GtrParams};

pub struct Gate {
    sr: f32,
    env: f32,
    gain: f32,
    thresh_lin: f32,
    env_release_coeff: f32,
    gain_release_coeff: f32,
}

impl Gate {
    pub fn new(sr: f32) -> Self {
        let mut g = Self {
            sr,
            env: 0.0,
            gain: 1.0,
            thresh_lin: db_to_gain(-60.0),
            env_release_coeff: 0.0,
            gain_release_coeff: 0.0,
        };
        g.update_params(sr, &GtrParams::default());
        g
    }

    pub fn reset(&mut self, sr: f32) {
        self.sr = sr;
        self.env = 0.0;
        self.gain = 1.0;
    }

    pub fn update_params(&mut self, sr: f32, p: &GtrParams) {
        self.sr = sr;
        self.thresh_lin = db_to_gain(p.gate_threshold.value());

        // envelope release - use the same time as gate release param
        let rel_s = (p.gate_release_ms.value() / 1000.0).max(1e-4);
        self.env_release_coeff = time_to_coeff(rel_s, self.sr);

        // gain smoothing, quicker than env
        self.gain_release_coeff = time_to_coeff(rel_s * 0.4, self.sr);
    }

    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        // Basic rectified envelope follower
        let level = x.abs();
        if level > self.env {
            self.env = level;
        } else {
            self.env *= self.env_release_coeff;
        }

        // Decide open/closed
        if self.env < self.thresh_lin {
            // close gate: fade gain towards 0
            self.gain *= self.gain_release_coeff;
        } else {
            // open gate: approach 1 quickly
            let open_coeff = 0.5; // 0..1, higher = faster
            self.gain += (1.0 - self.gain) * open_coeff;
        }

        flush_denormals(x * self.gain)
    }
}
