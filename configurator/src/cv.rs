use std::fmt;
use std::{error::Error, time::Duration};

use opencv::{
    core, highgui, imgproc,
    prelude::{Mat, MatTraitConstManual},
    videoio::VideoCaptureTraitConst,
    videoio::{self, VideoCaptureTrait},
};

#[derive(Debug)]
pub enum CameraError{
    InitializeError,
    CaptureError
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CameraError")
    }
}

impl Error for CameraError {}

#[derive(Default)]
pub struct Picture {
    inner: Mat,
}

impl Picture {
    pub fn mark(&mut self, x: usize, y: usize) -> Result<(), Box<dyn Error>> {
        imgproc::circle(
            &mut self.inner,
            core::Point::new(x as i32, y as i32),
            20,
            core::VecN::new(0.0, 0.0, 255.0, 255.0),
            2,
            imgproc::LINE_AA,
            0,
        )?;
        Ok(())
    }
}

impl From<Mat> for Picture {
    fn from(inner: Mat) -> Self {
        Self { inner }
    }
}

pub struct Camera {
    handle: videoio::VideoCapture,
}

impl Camera {
    pub fn new_default() -> Result<Self, Box<dyn Error>> {
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
        cam.set(videoio::CAP_PROP_BUFFERSIZE, 1.0)?;
        if videoio::VideoCapture::is_opened(&cam)? {
            Ok(Self { handle: cam })
        } else {
            Err(Box::new(CameraError::InitializeError))
        }
    }

    pub fn new_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut cam = videoio::VideoCapture::from_file(path, videoio::CAP_ANY)?; // 0 is the default camera
        cam.set(videoio::CAP_PROP_BUFFERSIZE, 1.0)?;
        if videoio::VideoCapture::is_opened(&cam)? {
            Ok(Self { handle: cam })
        } else {
            Err(Box::new(CameraError::InitializeError))
        }
    }

    pub fn capture(&mut self) -> Result<Picture, Box<dyn Error>> {
        let mut frame = Mat::default();
        self.handle.read(&mut frame)?;

        if frame.size()?.width > 0 {
            Ok(frame.into())
        } else {
            Err(Box::new(CameraError::CaptureError))
        }
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.handle.release().unwrap();
    }
}

pub struct Display {
    window_name: String,
}

impl Display {
    pub fn new(window_name: &str) -> Result<Self, Box<dyn Error>> {
        highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE)?;
        highgui::set_window_property(window_name, highgui::WND_PROP_TOPMOST, 1.0)?;

        Ok(Self {
            window_name: window_name.to_owned(),
        })
    }

    pub fn show(&self, picture: &Picture) -> Result<(), Box<dyn Error>> {
        highgui::imshow(&self.window_name, &picture.inner)?;
        Ok(())
    }

    pub fn wait_for(&self, duration: Duration) -> Result<i32, Box<dyn Error>> {
        Ok(highgui::wait_key(duration.as_millis() as i32)?)
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        highgui::destroy_window(&self.window_name).unwrap();
    }
}

pub fn find_light_from_diff(
    base_picture: &Picture,
    led_picture: &Picture,
) -> Result<Option<(usize, usize)>, Box<dyn Error>> {
    let mut base_gray = Mat::default();
    imgproc::cvt_color(
        &base_picture.inner,
        &mut base_gray,
        imgproc::COLOR_BGR2GRAY,
        0,
    )?;
    let mut led_gray = Mat::default();
    imgproc::cvt_color(
        &led_picture.inner,
        &mut led_gray,
        imgproc::COLOR_BGR2GRAY,
        0,
    )?;
    let mut diff = Mat::default();
    core::absdiff(&base_gray, &led_gray, &mut diff)?;

    // erode to remove the noise
    let mut eroded = Mat::default();
    let kernel = Mat::default();
    let anchor = core::Point::new(-1, -1); // default in C++ implementation
    let border_value = imgproc::morphology_default_border_value()?;
    imgproc::erode(
        &diff,
        &mut eroded,
        &kernel,
        anchor,
        1,
        core::BORDER_CONSTANT,
        border_value,
    )?;

    let mut max_loc = core::Point::default();
    let mut max_val: f64 = 0.0;
    opencv::core::min_max_loc(
        &eroded,
        None,
        Some(&mut max_val),
        None,
        Some(&mut max_loc),
        &Mat::default(),
    )?;

    println!("Max val: {}", max_val);
    if max_val < 80.0 {
        println!("Warning, low value detected: {}", max_val);
        return Ok(None);
    }

    Ok(Some((max_loc.x as usize, max_loc.y as usize)))
}
