use nih_plug::prelude::*;

#[derive(Params)]
pub struct GtrParams {
    /// Input trim before everything else (useful for hot DI vs quiet loops)
    #[id = "edm_in_gain"]
    pub input_gain: FloatParam,

    /// Overall amount of saturation/distortion. 0 = almost clean, 1 = heavy.
    #[id = "edm_drive"]
    pub drive: FloatParam,

    /// Dark/bright macro tone control. Internally you can implement this as a tilt EQ
    /// or map it to your presence/air filters.
    #[id = "edm_tone"]
    pub tone: FloatParam,

    /// Reverb/room amount. 0 = dry, 1 = max space.
    #[id = "edm_space"]
    pub space: FloatParam,

    /// Stereo widening. 0 = mono-ish, 0.5 ≈ neutral, 1 = very wide.
    #[id = "edm_width"]
    pub width: FloatParam,

    /// Final output level after limiter. Keep range modest so beginners
    /// don’t blow things up.
    #[id = "edm_out_gain"]
    pub output_gain: FloatParam,

    // ==========
    // ADVANCED
    // ==========
    /// Noise gate threshold in dBFS. More negative = more forgiving.
    #[id = "edm_gate_thresh"]
    pub gate_threshold: FloatParam,

    /// Gate release time in ms. Short = choppy, long = smoother tails.
    #[id = "edm_gate_release"]
    pub gate_release_ms: FloatParam,

    /// Amp "type" (all share one engine but different internal voicings).
    #[id = "edm_amp_model"]
    pub amp_model: EnumParam<AmpModel>,

    /// Pre-amp low-cut (IIR HPF) frequency, in Hz.
    #[id = "edm_low_cut_hz"]
    pub low_cut_hz: FloatParam,

    /// Upper-mids push. 0 = flat, 1 = very present / aggressive.
    #[id = "edm_presence"]
    pub presence: FloatParam,

    /// Top octave sheen. 0 = no extra air, 1 = very bright.
    #[id = "edm_air"]
    pub air: FloatParam,

    /// Cab/voicing model (IIR “cab curves” to start with).
    #[id = "edm_cab_model"]
    pub cab_model: EnumParam<CabModel>,

    /// Safety limiter toggle. On by default for beginners.
    #[id = "edm_limiter_on"]
    pub limiter_on: BoolParam,
}

#[derive(PartialEq, Eq, Clone, Copy, Enum)]
pub enum AmpModel {
    /// Wide, sparkly cleans for plucks and chorus guitars.
    CleanGlass,
    /// Tight modern crunch for rhythms and power chords.
    CrunchTight,
    /// Focused high-gain for leads and big EDM hooks.
    LeadEdm,
}

#[derive(PartialEq, Eq, Clone, Copy, Enum)]
pub enum CabModel {
    /// Modern tight cab: controlled lows, clear mids. Good EDM default.
    TightModern,
    /// Slightly rounder/warmer voicing.
    WarmVintage,
    /// Bright “pop” curve for cutting through dense mixes.
    BrightPop,
    /// Bypass cab shaping (for external cabs or creative uses).
    DiBypass,
}

impl Default for GtrParams {
    fn default() -> Self {
        Self {
            // ----- Simple / macro -----
            input_gain: FloatParam::new(
                "Input Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),

            drive: FloatParam::new(
                "Drive",
                0.55, // slightly crunchy by default
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            tone: FloatParam::new(
                "Tone",
                0.5, // neutral
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            space: FloatParam::new(
                "Space",
                0.25, // subtle ambience by default
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            width: FloatParam::new(
                "Width",
                0.6, // slightly wide, mix-friendly
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            output_gain: FloatParam::new(
                "Output Gain",
                0.0,
                FloatRange::Linear {
                    min: -18.0,
                    max: 6.0,
                },
            )
            .with_unit(" dB"),

            // ----- Advanced -----
            gate_threshold: FloatParam::new(
                "Gate Threshold",
                -50.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB"),

            gate_release_ms: FloatParam::new(
                "Gate Release",
                80.0,
                FloatRange::Skewed {
                    min: 10.0,
                    max: 400.0,
                    factor: 0.4, // more resolution at short times
                },
            )
            .with_unit(" ms"),

            amp_model: EnumParam::new("Amp Model", AmpModel::CrunchTight),

            low_cut_hz: FloatParam::new(
                "Low Cut",
                110.0, // keeps guitars out of the sub/bass lane
                FloatRange::Skewed {
                    min: 40.0,
                    max: 200.0,
                    factor: 0.3, // quasi-log
                },
            )
            .with_unit(" Hz"),

            presence: FloatParam::new(
                "Presence",
                0.5, // neutral
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            air: FloatParam::new(
                "Air",
                0.3, // a bit of top-end sheen by default
                FloatRange::Linear { min: 0.0, max: 1.0 },
            ),

            cab_model: EnumParam::new("Cab", CabModel::TightModern),

            limiter_on: BoolParam::new("Limiter", true),
        }
    }
}

impl GtrParams {
    #[inline]
    pub fn input_linear(&self) -> f32 {
        db_to_gain(self.input_gain.value())
    }

    #[inline]
    pub fn output_linear(&self) -> f32 {
        db_to_gain(self.output_gain.value())
    }
}

/// Convert dB value to linear gain. Use this in your DSP code.
#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    // ln(10) / 20 ≈ 0.115129
    (db * 0.115129f32).exp()
}
