use crate::dsp::{fast_tanh, flush_denormals};
use crate::params::{AmpModel, GtrParams};

pub struct Amp {
    sr: f32,
}

impl Amp {
    pub fn new(sr: f32) -> Self {
        Self { sr }
    }

    pub fn reset(&mut self, sr: f32) {
        self.sr = sr;
    }

    #[inline]
    pub fn process_sample(&mut self, x: f32, p: &GtrParams) -> f32 {
        let drive = p.drive.value().clamp(0.0, 1.0);
        let model = p.amp_model.value();

        // Map drive + model to pre-gain and mix amounts
        let (pre_gain, sat_strength, post_atten) = match model {
            AmpModel::CleanGlass => {
                // Mostly clean, some gentle color
                let pre = 1.0 + 3.0 * drive; // up to ~4x
                let sat = 0.7 + 0.3 * drive; // slightly more drive at max
                let post = 0.9;
                (pre, sat, post)
            }
            AmpModel::CrunchTight => {
                // Rock/modern crunch
                let pre = 1.5 + 6.0 * drive; // up to ~7.5x
                let sat = 0.9 + 0.5 * drive;
                let post = 0.7;
                (pre, sat, post)
            }
            AmpModel::LeadEdm => {
                // Focused high-gain
                let pre = 2.0 + 10.0 * drive; // up to ~12x
                let sat = 1.1 + 0.6 * drive;
                let post = 0.6;
                (pre, sat, post)
            }
        };

        // Pre-boost into waveshaper
        let pre = x * pre_gain;
        let shaped = fast_tanh(pre * sat_strength);

        // Wet/dry mix based on drive
        let wet_mix = 0.2 + 0.7 * drive; // 0.2..0.9
        let dry_mix = 1.0 - wet_mix;
        let y = pre * dry_mix + shaped * wet_mix;

        flush_denormals(y * post_atten)
    }
}
