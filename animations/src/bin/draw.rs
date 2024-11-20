use animation_api::Animation;
use animation_utils::{decorators::BrightnessControlled, EnumSchema, Schema};
use lightfx::Color;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Default, Serialize, Deserialize, EnumSchema, PartialEq, Eq)]
enum Mode {
    #[schema_variant(name = "Pencil")]
    #[default]
    Pencil,

    #[schema_variant(name = "Eraser")]
    Eraser,

    #[schema_variant(name = "Watch")]
    Watching,
}

#[derive(Clone, Serialize, Deserialize, Schema)]
pub struct Parameters {
    #[schema_field(name = "Mode", enum_options)]
    mode: Mode,

    #[schema_field(name = "Color", color)]
    color: Color,

    #[schema_field(name = "Brush size", number(min = 0.0, max = 1.0, step = 0.05))]
    brush_size: f32,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            mode: Default::default(),
            color: Color::white(),
            brush_size: 0.1,
        }
    }
}
#[animation_utils::plugin]
pub struct Draw {
    points: Vec<nalgebra::Vector3<f32>>,
    canvas: Vec<Color>,
    drawing: bool,
    last_ray_origin: Option<nalgebra::Vector3<f32>>,
    last_ray_direction: Option<nalgebra::Vector3<f32>>,
    parameters: Parameters,
}

impl Animation for Draw {
    type Parameters = Parameters;
    type Wrapped = BrightnessControlled<Self>;

    fn new(points: Vec<(f64, f64, f64)>) -> Self {
        let len = points.len();
        Self {
            points: points
                .into_iter()
                .map(|(x, y, z)| nalgebra::Vector3::new(x as f32, y as f32, z as f32))
                .collect(),
            canvas: vec![Color::black(); len],
            drawing: false,
            last_ray_origin: None,
            last_ray_direction: None,
            parameters: Default::default(),
        }
    }

    fn set_parameters(&mut self, parameters: Self::Parameters) {
        self.parameters = parameters;
    }

    fn get_parameters(&self) -> Self::Parameters
    where
        Self::Parameters: Default,
    {
        self.parameters.clone()
    }

    fn update(&mut self, _delta: f64) {}

    fn render(&self) -> lightfx::Frame {
        let mut canvas = self.canvas.clone();
        if self.parameters.mode != Mode::Watching {
            if let Some((origin, direction)) = self.last_ray_origin.zip(self.last_ray_direction) {
                light_up(
                    &self.points,
                    &mut canvas,
                    origin,
                    direction,
                    &self.parameters,
                )
            }
        }
        lightfx::Frame::from_vec(canvas)
    }

    fn on_event(&mut self, event: animation_api::event::Event) {
        match event {
            animation_api::event::Event::MouseMove {
                ray_origin,
                ray_direction,
            } => {
                let ray_origin = nalgebra::Vector3::from_iterator(ray_origin);
                let ray_direction = nalgebra::Vector3::from_iterator(ray_direction);

                self.last_ray_origin = Some(ray_origin);
                self.last_ray_direction = Some(ray_direction);

                if !self.drawing {
                    return;
                }

                light_up(
                    &self.points,
                    &mut self.canvas,
                    ray_origin,
                    ray_direction,
                    &self.parameters,
                );
            }
            animation_api::event::Event::MouseDown => {
                self.drawing = true;
            }
            animation_api::event::Event::MouseUp => {
                self.drawing = false;
            }
            _ => (),
        }
    }
}

fn light_up(
    points: &[nalgebra::Vector3<f32>],
    pixels: &mut [Color],
    ray_origin: nalgebra::Vector3<f32>,
    ray_direction: nalgebra::Vector3<f32>,
    parameters: &Parameters,
) {
    let a = ray_origin;
    let n = ray_direction.normalize();
    points.iter().zip(pixels.iter_mut()).for_each(|(p, pixel)| {
        let distance = (a - p - (a - p).dot(&n) * n).norm();
        if distance < parameters.brush_size / 2.0 {
            match parameters.mode {
                Mode::Pencil => *pixel = parameters.color,
                Mode::Eraser => *pixel = Color::black(),
                _ => (),
            }
        }
    });
}
