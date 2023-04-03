use serde::{Deserialize, Serialize};

use crate::Color;

#[derive(Clone, Serialize, Deserialize)]
pub struct Frame {
    pixels: Vec<Color>,
}

impl Frame {
    pub fn new(number_of_lights: usize, color: Color) -> Self {
        Self {
            pixels: vec![color; number_of_lights],
        }
    }
    pub fn new_black(number_of_lights: usize) -> Self {
        Self::new(number_of_lights, Color::black())
    }

    pub fn set_pixel(&mut self, index: usize, color: Color) {
        self.pixels[index] = color;
    }

    pub fn with_pixel(mut self, index: usize, color: Color) -> Self {
        self.pixels[index] = color;
        self
    }

    pub fn pixels_iter(&self) -> impl Iterator<Item = &Color> {
        self.pixels.iter()
    }
}

impl<T> From<T> for Frame
where
    T: Iterator<Item = Color>,
{
    fn from(iter: T) -> Self {
        Self {
            pixels: iter.collect(),
        }
    }
}
