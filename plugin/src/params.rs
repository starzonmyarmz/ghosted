use crate::engine::{Controls, Settings, Voices};
use nice_plug::prelude::*;

#[derive(Enum, Clone, Copy, PartialEq, Debug)]
pub enum VoicesParam {
    #[name = "2"]
    Two,
    #[name = "4"]
    Four,
}

#[derive(Params)]
pub struct GhostedParams {
    #[id = "haunt"]
    pub haunt: FloatParam,
    #[id = "distance"]
    pub distance: FloatParam,
    #[id = "drift"]
    pub drift: FloatParam,
    #[id = "spook"]
    pub spook: FloatParam,
    #[id = "voices"]
    pub voices: EnumParam<VoicesParam>,
}

fn percent(name: &str, default: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 1.0 })
        .with_unit("%")
        .with_value_to_string(formatters::v2s_f32_percentage(0))
        .with_string_to_value(formatters::s2v_f32_percentage())
}

impl Default for GhostedParams {
    fn default() -> Self {
        let c = Controls::default();
        Self {
            haunt: percent("Haunt", c.haunt as f32),
            distance: FloatParam::new(
                "Distance",
                c.distance_ms as f32,
                FloatRange::Skewed { min: 5.0, max: 150.0, factor: FloatRange::skew_factor(-1.0) },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
            drift: percent("Drift", c.drift as f32),
            spook: percent("Spook", c.spook as f32),
            voices: EnumParam::new("Voices", VoicesParam::Four),
        }
    }
}

impl GhostedParams {
    pub fn settings(&self) -> Settings {
        Controls {
            haunt: self.haunt.value() as f64,
            distance_ms: self.distance.value() as f64,
            drift: self.drift.value() as f64,
            spook: self.spook.value() as f64,
            voices: match self.voices.value() {
                VoicesParam::Two => Voices::Two,
                VoicesParam::Four => Voices::Four,
            },
        }
        .settings()
    }
}
