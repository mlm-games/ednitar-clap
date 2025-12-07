use crate::dsp::filters::{OnePoleHp, TiltEq};
use crate::dsp::flush_denormals;
use crate::params::{CabModel, GtrParams};

pub struct Cab {
    sr: f32,
    tilt: TiltEq,
    presence_hp: OnePoleHp,
    air_hp: OnePoleHp,
}

impl Cab {
    pub fn new(sr: f32) -> Self {
        let mut c = Self {
            sr,
            tilt: TiltEq::new(),
            presence_hp: OnePoleHp::new(),
            air_hp: OnePoleHp::new(),
        };
        c.reset(sr);
        c
    }

    pub fn reset(&mut self, sr: f32) {
        self.sr = sr;
        self.tilt = TiltEq::new();
        self.tilt.set_pivot(self.sr, 1600.0); // pivot for tilt EQ
        self.presence_hp = OnePoleHp::new();
        self.presence_hp.set_cutoff(self.sr, 3000.0);
        self.air_hp = OnePoleHp::new();
        self.air_hp.set_cutoff(self.sr, 8000.0);
    }

    pub fn update_params(&mut self, sr: f32) {
        self.sr = sr;
        // If you want sample-rate dependent recalcs, do them here.
        self.tilt.set_pivot(self.sr, 1600.0);
        self.presence_hp.set_cutoff(self.sr, 3000.0);
        self.air_hp.set_cutoff(self.sr, 8000.0);
    }

    #[inline]
    pub fn process_sample(&mut self, x: f32, p: &GtrParams) -> f32 {
        let mut y = x;

        let tone = p.tone.value(); // 0..1
        let pres = p.presence.value(); // 0..1
        let air = p.air.value(); // 0..1
        let cab = p.cab_model.value();

        // Cab-dependent tone bias
        let tone_bias = match cab {
            CabModel::TightModern => 0.1,  // slightly brighter default
            CabModel::WarmVintage => -0.1, // slightly darker
            CabModel::BrightPop => 0.25,   // quite bright
            CabModel::DiBypass => 0.0,
        };

        let tone_amt = (tone + tone_bias).clamp(0.0, 1.0);
        let tilt = tone_amt * 2.0 - 1.0; // -1..1

        // Global cab tilt EQ
        if !matches!(cab, CabModel::DiBypass) {
            y = self.tilt.process(y, tilt);
        }

        // Presence: emphasis on upper mids via high-pass at ~3k
        if pres > 0.0 && !matches!(cab, CabModel::DiBypass) {
            let base = 1.0 + 2.0 * pres; // up to ~+6 dB subjective
            let hi_mid = self.presence_hp.process(y);
            y += hi_mid * (base - 1.0);
        }

        // Air: top octave via high-pass at ~8k
        if air > 0.0 && !matches!(cab, CabModel::DiBypass) {
            let base = 1.0 + 3.0 * air;
            let hi_hi = self.air_hp.process(y);
            y += hi_hi * (base - 1.0);
        }

        flush_denormals(y)
    }
}
