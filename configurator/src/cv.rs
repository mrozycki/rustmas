use std::fmt;
use std::{error::Error, time::Duration};

use opencv::{
    core, highgui, imgproc,
    prelude::{Mat, MatTraitConstManual},
    videoio::VideoCaptureTraitConst,
    videoio::{self, VideoCaptureTrait},
};

#[derive(Debug)]
pub struct CameraError();

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
        let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
        if videoio::VideoCapture::is_opened(&cam)? {
            Ok(Self { handle: cam })
        } else {
            Err(Box::new(CameraError()))
        }
    }

    pub fn new_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let cam = videoio::VideoCapture::from_file(path, videoio::CAP_ANY)?; // 0 is the default camera
        if videoio::VideoCapture::is_opened(&cam)? {
            Ok(Self { handle: cam })
        } else {
            Err(Box::new(CameraError()))
        }
    }

    pub fn capture(&mut self) -> Result<Picture, Box<dyn Error>> {
        let mut frame = Mat::default();
        self.handle.read(&mut frame)?;

        if frame.size()?.width > 0 {
            Ok(frame.into())
        } else {
            Err(Box::new(CameraError()))
        }
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.handle.release().unwrap();
    }
}

pub fn find_light(picture: &Picture) -> Result<(usize, usize), Box<dyn Error>> {
    let mut hsv = Mat::default();
    imgproc::cvt_color(&picture.inner, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let lower = core::Scalar::from((0.0, 0.0, 255.0));
    let upper = core::Scalar::from((5.0, 128.0, 255.0));
    let mut mask = Mat::default();
    opencv::core::in_range(&hsv, &lower, &upper, &mut mask)?;

    let mut max_loc = core::Point::default();
    opencv::core::min_max_loc(&mask, None, None, None, Some(&mut max_loc), &mask)?;

    Ok((max_loc.x as usize, max_loc.y as usize))
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
