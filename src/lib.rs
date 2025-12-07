mod dsp;
mod params;
mod presets;

use nih_plug::prelude::*;
use std::num::NonZeroU32;
use std::sync::Arc;

use dsp::{GuitarChain, StereoFx};
use params::GtrParams;

pub struct EdmGtr {
    params: Arc<GtrParams>,
    sample_rate: f32,
    chain_l: GuitarChain,
    chain_r: GuitarChain,
    fx: StereoFx,
}

impl Default for EdmGtr {
    fn default() -> Self {
        let sr = 44100.0;
        Self {
            params: Arc::new(GtrParams::default()),
            sample_rate: sr,
            chain_l: GuitarChain::new(sr),
            chain_r: GuitarChain::new(sr),
            fx: StereoFx::new(sr),
        }
    }
}

impl Plugin for EdmGtr {
    const NAME: &'static str = "EdmGtr";
    const VENDOR: &'static str = "me";
    const URL: &'static str = "https://website.com";
    const EMAIL: &'static str = "me@website.com";
    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        // Stereo in / stereo out
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _io: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _ctx: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        self.chain_l.reset(self.sample_rate);
        self.chain_r.reset(self.sample_rate);
        self.fx.reset(self.sample_rate);

        true
    }

    fn reset(&mut self) {
        self.chain_l.reset(self.sample_rate);
        self.chain_r.reset(self.sample_rate);
        self.fx.reset(self.sample_rate);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer<'_>,
        _aux: &mut AuxiliaryBuffers<'_>,
        _ctx: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let params = self.params.clone();

        // Update per-block coefficients
        self.chain_l.update_params(params.as_ref());
        self.chain_r.update_params(params.as_ref());

        for mut frame in buffer.iter_samples() {
            let mut it = frame.iter_mut();
            if let Some(l_s) = it.next() {
                let r_s_opt = it.next();

                let in_l = *l_s;
                let in_r = r_s_opt.as_deref().copied().unwrap_or(in_l);

                let dry_l = self.chain_l.process_sample(in_l, params.as_ref());
                let dry_r = self.chain_r.process_sample(in_r, params.as_ref());

                let (out_l, out_r) = self.fx.process_frame(dry_l, dry_r, params.as_ref());

                *l_s = out_l;
                if let Some(r_s) = r_s_opt {
                    *r_s = out_r;
                }
            }
        }

        ProcessStatus::Normal
    }
}

// CLAP metadata
impl ClapPlugin for EdmGtr {
    const CLAP_ID: &'static str = "dev.example.edmgtr";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Minimal EDM guitar CLAP effect (stereo, IIR cab).");
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Distortion,
    ];

    const CLAP_POLY_MODULATION_CONFIG: Option<PolyModulationConfig> = None;

    fn remote_controls(&self, _context: &mut impl RemoteControlsContext) {}

    const CLAP_MANUAL_URL: Option<&'static str> = Some("Not yet");
    const CLAP_SUPPORT_URL: Option<&'static str> = Some("Not yet");
}

nih_export_clap!(EdmGtr);
