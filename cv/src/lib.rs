use std::fmt;
use std::sync::{Arc, Mutex};
use std::{error::Error, time::Duration};

use log::debug;
use opencv::{
    core, highgui, imgproc,
    prelude::{Mat, MatTraitConstManual},
    videoio::{self, VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst},
};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithConfidence<T> {
    pub inner: T,
    pub confidence: f64,
}

impl<T> WithConfidence<T> {
    pub fn confident(&self) -> bool {
        return self.confidence > 0.3;
    }
}

#[derive(Debug)]
pub enum CameraError {
    InitializeError,
    CaptureError,
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

    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        debug!("Writing file: {}", filename);
        opencv::imgcodecs::imwrite(filename, &self.inner, &opencv::core::Vector::default())?;
        Ok(())
    }
}

impl From<Mat> for Picture {
    fn from(inner: Mat) -> Self {
        Self { inner }
    }
}

pub struct Camera {
    camera_handle: Arc<Mutex<videoio::VideoCapture>>,
    _grabber_thread_join: JoinHandle<()>,
}

impl Camera {
    pub fn new_from_video_capture(mut capture: VideoCapture) -> Result<Self, Box<dyn Error>> {
        capture.set(videoio::CAP_PROP_BUFFERSIZE, 1.0)?;

        if !videoio::VideoCapture::is_opened(&capture)? {
            return Err(Box::new(CameraError::InitializeError));
        }
        let camera_handle = Arc::new(Mutex::new(capture));
        let camera_handle_clone = camera_handle.clone();
        let _grabber_thread_join = tokio::spawn(async move {
            loop {
                let _ = camera_handle_clone.lock().unwrap().grab();
                tokio::time::sleep(Duration::from_millis(30)).await;
            }
        });
        Ok(Self {
            camera_handle,
            _grabber_thread_join,
        })
    }
    pub fn new_default() -> Result<Self, Box<dyn Error>> {
        let cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?; // 0 is the default camera
        Self::new_from_video_capture(cam)
    }

    pub fn new_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let cam = videoio::VideoCapture::from_file(path, videoio::CAP_ANY)?; // 0 is the default camera
        Self::new_from_video_capture(cam)
    }

    pub fn capture(&mut self) -> Result<Picture, Box<dyn Error>> {
        let mut frame = Mat::default();
        let mut camera_handle = self.camera_handle.lock().unwrap();
        camera_handle.grab()?;
        camera_handle.read(&mut frame)?;

        if frame.size()?.width > 0 {
            Ok(frame.into())
        } else {
            Err(Box::new(CameraError::CaptureError))
        }
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        self.camera_handle.lock().unwrap().release().unwrap();
    }
}

pub struct Display {
    window_name: String,
}

#[allow(unused)]
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
) -> Result<WithConfidence<(usize, usize)>, Box<dyn Error>> {
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
    if max_loc.x < 0 || max_loc.y < 0 {
        // OpenCV might return (-1,-1) if it can't find anything
        return Ok(WithConfidence::<(usize, usize)> {
            inner: (0, 0),
            confidence: f64::NEG_INFINITY,
        });
    }

    Ok(WithConfidence::<(usize, usize)> {
        inner: (max_loc.x as usize, max_loc.y as usize),
        confidence: max_val / 255.0,
    })
}
