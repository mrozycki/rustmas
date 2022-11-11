use super::Animation;
use rustmas_light_client as client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Parameters {
    x: f64,
    y: f64,
    z: f64,
}

pub struct RainbowSphere {
    points: Vec<(f64, f64, f64)>,
    points_radius: Vec<f64>,
    parameters: Parameters,
}

impl RainbowSphere {
    fn reset(&mut self) {
        self.points_radius = self
            .points
            .iter()
            .map(|(x, y, z)| {
                ((x - self.parameters.x).powi(2)
                    + (y - self.parameters.y).powi(2)
                    + (z - self.parameters.z).powi(2))
                .sqrt()
            })
            .collect();
    }

    pub fn new(points: &Vec<(f64, f64, f64)>) -> Self {
        let mut result = Self {
            points_radius: vec![],
            points: points.clone(),
            parameters: Parameters {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };
        result.reset();
        result
    }
}

impl Animation for RainbowSphere {
    fn frame(&mut self, time: f64) -> client::Frame {
        self.points_radius
            .iter()
            .map(|r| client::Color::hsv(r - time, 1.0, 1.0))
            .into()
    }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "x": {
                "type": "number",
                "min": -1,
                "max": 1
            },
            "y": {
                "type": "number",
                "min": -1,
                "max": 1
            },
            "z": {
                "type": "number",
                "min": -1,
                "max": 1
            },
        })
    }

    fn set_parameters(
        &mut self,
        parameters: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.parameters = serde_json::from_value(parameters)?;
        self.reset();
        Ok(())
    }
}
