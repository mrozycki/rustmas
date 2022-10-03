use core::fmt;
use std::cell::{Ref, RefCell};

#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn gray(shade: u8) -> Self {
        Self::rgb(shade, shade, shade)
    }

    pub fn black() -> Self {
        Self::gray(0)
    }

    pub fn white() -> Self {
        Self::gray(255)
    }
}

#[derive(Clone)]
pub struct Frame {
    pixels: Vec<Color>,
}

impl Frame {
    pub fn new(light_count: usize, color: Color) -> Self {
        Self {
            pixels: vec![color; light_count],
        }
    }
    pub fn new_black(light_count: usize) -> Self {
        Self::new(light_count, Color::black())
    }

    pub fn set_pixel(&mut self, index: usize, color: Color) {
        self.pixels[index] = color;
    }

    pub fn with_pixel(mut self, index: usize, color: Color) -> Self {
        self.pixels[index] = color;
        self
    }
}

impl From<Vec<Color>> for Frame {
    fn from(pixels: Vec<Color>) -> Self {
        Self { pixels }
    }
}

#[derive(Debug)]
pub enum LightClientError {
    Unlikely,
}

impl fmt::Display for LightClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for LightClientError {}

pub trait LightClient {
    fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError>;
}

pub struct MockLightClient {
    frames: RefCell<Vec<Frame>>,
}

impl MockLightClient {
    pub fn new() -> Self {
        Self {
            frames: RefCell::new(Vec::new()),
        }
    }

    pub fn get_frames(&self) -> Ref<Vec<Frame>> {
        self.frames.borrow()
    }
}

impl LightClient for MockLightClient {
    fn display_frame(&self, frame: &Frame) -> Result<(), LightClientError> {
        self.frames.borrow_mut().push(frame.clone());
        Ok(())
    }
}
