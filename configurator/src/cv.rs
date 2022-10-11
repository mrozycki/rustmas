use std::error::Error;
use std::fmt;

use opencv::{
    core, imgproc,
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

pub struct Camera {
    handle: videoio::VideoCapture,
}

pub type Picture = Mat;

impl Camera {
    pub fn new_default() -> Result<Self, Box<dyn Error>> {
        opencv::opencv_branch_32! {
            let cam = videoio::VideoCapture::new_default(0)?; // 0 is the default camera
        }
        opencv::not_opencv_branch_32! {
            let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
        }
        let opened = videoio::VideoCapture::is_opened(&cam)?;
        if opened {
            Ok(Self { handle: cam })
        } else {
            Err(Box::new(CameraError()))
        }
    }

    pub fn capture(&mut self) -> Result<Picture, Box<dyn Error>> {
        let mut frame = Mat::default();
        self.handle.read(&mut frame)?;

        if frame.size()?.width > 0 {
            Ok(frame)
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
    imgproc::cvt_color(picture, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let lower = core::Scalar::from((0.0, 0.0, 255.0));
    let upper = core::Scalar::from((5.0, 128.0, 255.0));
    let mut mask = Mat::default();
    opencv::core::in_range(&hsv, &lower, &upper, &mut mask)?;

    let mut max_loc = core::Point::default();
    opencv::core::min_max_loc(&mask, None, None, None, Some(&mut max_loc), &mask)?;

    Ok((max_loc.x as usize, max_loc.y as usize))
}
