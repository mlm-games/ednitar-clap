// src/dsp/chain.rs
use crate::dsp::amp::Amp;
use crate::dsp::cab::Cab;
use crate::dsp::filters::OnePoleHp;
use crate::dsp::gate::Gate;
use crate::params::GtrParams;

pub struct GuitarChain {
    sr: f32,
    gate: Gate,
    pre_lowcut: OnePoleHp,
    amp: Amp,
    cab: Cab,
}

impl GuitarChain {
    pub fn new(sr: f32) -> Self {
        let mut chain = Self {
            sr,
            gate: Gate::new(sr),
            pre_lowcut: OnePoleHp::new(),
            amp: Amp::new(sr),
            cab: Cab::new(sr),
        };
        chain.reset(sr);
        chain
    }

    pub fn reset(&mut self, sr: f32) {
        self.sr = sr;
        self.gate.reset(sr);
        self.pre_lowcut = OnePoleHp::new();
        self.pre_lowcut.set_cutoff(self.sr, 100.0);
        self.amp.reset(sr);
        self.cab.reset(sr);
    }

    /// Called once per processing block to update filter coefficients etc.
    pub fn update_params(&mut self, p: &GtrParams) {
        self.gate.update_params(self.sr, p);
        // user-controlled low cut
        self.pre_lowcut.set_cutoff(self.sr, p.low_cut_hz.value());
        self.cab.update_params(self.sr);
    }

    #[inline]
    pub fn process_sample(&mut self, x: f32, p: &GtrParams) -> f32 {
        // Input gain before anything
        let mut s = x * p.input_linear();

        // Noise gate
        s = self.gate.process_sample(s);

        // Pre-amp low cut
        s = self.pre_lowcut.process(s);

        // Amp / distortion
        s = self.amp.process_sample(s, p);

        // Cab voicing + tone, presence, air
        s = self.cab.process_sample(s, p);

        s
    }
}
