//! Basic repose GUI for Ednitar (UI thread only, could be a lot better but is mostly for demonstration).

use nih_plug::prelude::*;
use repose_core::prelude::*;
use repose_material::material3::{Slider, SliderConfig, Switch, SwitchConfig};
use repose_audui::nih_plug::{create_repose_editor, ReposeEditorState};
use repose_audui::param_widgets::{panel, section_title};
use repose_ui::{Column, Row, Text, TextStyle, ViewExt};
use std::sync::Arc;

use crate::params::{AmpModel, CabModel, GtrParams};

pub fn create_editor(params: Arc<GtrParams>) -> Option<Box<dyn Editor>> {
    let state = ReposeEditorState::new(560, 640);
    Some(create_repose_editor(
        state,
        "Ednitar",
        {
            let params = params.clone();
            move |_sched, _ctx, gui| ednitar_ui(params.clone(), gui.clone())
        },
        None,
    ))
}

fn ednitar_ui(params: Arc<GtrParams>, gui: Arc<dyn GuiContext>) -> View {
    // Read plain values every compose (automation-safe).
    let input_gain = params.input_gain.value();
    let drive = params.drive.value();
    let tone = params.tone.value();
    let space = params.space.value();
    let width = params.width.value();
    let output_gain = params.output_gain.value();
    let gate_threshold = params.gate_threshold.value();
    let gate_release = params.gate_release_ms.value();
    let low_cut = params.low_cut_hz.value();
    let presence = params.presence.value();
    let air = params.air.value();
    let limiter_on = params.limiter_on.value();
    let amp = params.amp_model.value();
    let cab = params.cab_model.value();

    let setter_gui = gui.clone();

    // Helper: continuous float gesture (begin/set/end around drag).
    // host should see changes via ParamSetter.
    macro_rules! float_slider {
        ($field:ident, $val:expr, $range:expr, $step:expr) => {{
            let inner = params.clone();
            let g = setter_gui.clone();
            Slider(
                $val,
                $range,
                $step,
                move |v| {
                    let setter = ParamSetter::new(g.as_ref());
                    setter.begin_set_parameter(&inner.$field);
                    setter.set_parameter(&inner.$field, v);
                    setter.end_set_parameter(&inner.$field);
                },
                SliderConfig::default(),
            )
        }};
    }

    panel(vec![
        section_title("Ednitar"),
        Text("EDM guitar macro chain").size(13.0),
        labeled("Input Gain", format!("{input_gain:.1} dB"), float_slider!(
            input_gain, input_gain, (-24.0, 24.0), Some(0.1)
        )),
        labeled("Drive", format!("{:.0}%", drive * 100.0), float_slider!(
            drive, drive, (0.0, 1.0), Some(0.01)
        )),
        labeled("Tone", format!("{:.0}%", tone * 100.0), float_slider!(
            tone, tone, (0.0, 1.0), Some(0.01)
        )),
        labeled("Space", format!("{:.0}%", space * 100.0), float_slider!(
            space, space, (0.0, 1.0), Some(0.01)
        )),
        labeled("Width", format!("{:.0}%", width * 100.0), float_slider!(
            width, width, (0.0, 1.0), Some(0.01)
        )),
        labeled("Output Gain", format!("{output_gain:.1} dB"), float_slider!(
            output_gain, output_gain, (-18.0, 6.0), Some(0.1)
        )),
        section_title("Advanced"),
        labeled("Gate Thresh", format!("{gate_threshold:.0} dB"), float_slider!(
            gate_threshold, gate_threshold, (-60.0, 0.0), Some(0.5)
        )),
        labeled("Gate Release", format!("{gate_release:.0} ms"), float_slider!(
            gate_release_ms, gate_release, (10.0, 400.0), Some(1.0)
        )),
        labeled("Low Cut", format!("{low_cut:.0} Hz"), float_slider!(
            low_cut_hz, low_cut, (40.0, 200.0), Some(1.0)
        )),
        labeled("Presence", format!("{:.0}%", presence * 100.0), float_slider!(
            presence, presence, (0.0, 1.0), Some(0.01)
        )),
        labeled("Air", format!("{:.0}%", air * 100.0), float_slider!(
            air, air, (0.0, 1.0), Some(0.01)
        )),
        amp_row(params.clone(), gui.clone(), amp),
        cab_row(params.clone(), gui.clone(), cab),
        {
            let g = gui.clone();
            Row(Modifier::new().gap(12.0).fill_max_width()).child((
                Text("Limiter").size(14.0),
                Switch(
                    limiter_on,
                    move |v| {
                        let setter = ParamSetter::new(g.as_ref());
                        setter.begin_set_parameter(&params.limiter_on);
                        setter.set_parameter(&params.limiter_on, v);
                        setter.end_set_parameter(&params.limiter_on);
                    },
                    SwitchConfig::default(),
                ),
            ))
        },
    ])
}

fn labeled(name: &str, value_text: String, slider: View) -> View {
    Column(Modifier::new().gap(4.0).fill_max_width()).child((
        Row(Modifier::new().gap(8.0).fill_max_width()).child((
            Text(name.to_string()).size(13.0),
            Text(value_text).size(12.0),
        )),
        slider,
    ))
}

fn amp_row(params: Arc<GtrParams>, gui: Arc<dyn GuiContext>, current: AmpModel) -> View {
    let models = [
        (AmpModel::CleanGlass, "Clean"),
        (AmpModel::CrunchTight, "Crunch"),
        (AmpModel::LeadEdm, "Lead"),
    ];
    enum_picker("Amp", models, current, move |m| {
        let setter = ParamSetter::new(gui.as_ref());
        setter.begin_set_parameter(&params.amp_model);
        setter.set_parameter(&params.amp_model, m);
        setter.end_set_parameter(&params.amp_model);
    })
}

fn cab_row(params: Arc<GtrParams>, gui: Arc<dyn GuiContext>, current: CabModel) -> View {
    let models = [
        (CabModel::TightModern, "Tight"),
        (CabModel::WarmVintage, "Warm"),
        (CabModel::BrightPop, "Bright"),
        (CabModel::DiBypass, "DI"),
    ];
    enum_picker("Cab", models, current, move |m| {
        let setter = ParamSetter::new(gui.as_ref());
        setter.begin_set_parameter(&params.cab_model);
        setter.set_parameter(&params.cab_model, m);
        setter.end_set_parameter(&params.cab_model);
    })
}

fn enum_picker<T: Copy + PartialEq + 'static>(
    title: &str,
    items: impl IntoIterator<Item = (T, &'static str)>,
    current: T,
    on_pick: impl Fn(T) + Clone + 'static,
) -> View {
    let mut kids: Vec<View> = vec![Text(title.to_string()).size(13.0)];
    for (val, label) in items {
        let selected = val == current;
        let on_pick = on_pick.clone();
        kids.push(
            repose_ui::Box(
                Modifier::new()
                    .padding(8.0)
                    .clip_rounded(6.0)
                    .background(if selected {
                        Color::from_rgba(70, 120, 200, 255)
                    } else {
                        Color::from_rgba(50, 50, 58, 255)
                    })
                    .on_click(move || on_pick(val)),
            )
            .child(Text(label.to_string()).size(12.0)),
        );
    }
    Column(Modifier::new().gap(6.0).fill_max_width()).child(kids)
}