use animation_api::event::Event;
use animation_api::Animation;
use animation_utils::decorators::{BrightnessControlled, SpeedControlled};
use lightfx::Color;
use serde_json::json;

#[animation_utils::plugin]
pub struct AudioVisualizer {
    points: Vec<(f64, f64, f64)>,
    bands: Vec<f32>,
}

impl AudioVisualizer {
    pub fn create(points: Vec<(f64, f64, f64)>) -> impl Animation {
        SpeedControlled::new(BrightnessControlled::new(Self {
            points,
            bands: vec![0.0],
        }))
    }
}

impl Animation for AudioVisualizer {
    type Parameters = ();

    fn on_event(&mut self, event: Event) {
        if let Event::FftEvent { bands } = event {
            self.bands = bands;
        }
    }

    fn render(&self) -> lightfx::Frame {
        self.points
            .iter()
            .map(|(x, y, _)| {
                let band_width = 2.0 / self.bands.len() as f64;
                let band_index =
                    ((x + 1.0).div_euclid(band_width) as usize).clamp(0, self.bands.len() - 1);
                let band_value = self.bands[band_index].clamp(0.0, 1.0);
                let y = (*y as f32 + 1.0) / 2.0;
                if y < band_value {
                    Color::hsv((x + 1.0) / 2.0, 1.0, 1.0)
                } else {
                    Color::black()
                }
            })
            .into()
    }

    fn animation_name(&self) -> &str {
        "Audio Visualizer"
    }
}
