use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{error::Error, time::Duration};

use log::debug;
use opencv::core::{MatExprTraitConst, CV_32F};
use opencv::imgproc::COLOR_GRAY2RGB;
use opencv::{
    core, highgui, imgproc,
    prelude::{Mat, MatTraitConst},
    videoio::{self, VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst},
};
use tokio::task::JoinHandle;

use crate::capture::WithConfidence;

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
    pub fn mark(&mut self, coords: &WithConfidence<(usize, usize)>) -> Result<(), Box<dyn Error>> {
        let color = if coords.confident() {
            core::VecN::new(0.0, 255.0, 0.0, 255.0) // green
        } else {
            core::VecN::new(0.0, 0.0, 255.0, 255.0) // red
        };
        imgproc::circle(
            &mut self.inner,
            core::Point::new(coords.inner.0 as i32, coords.inner.1 as i32),
            20,
            color,
            2,
            imgproc::LINE_AA,
            0,
        )?;
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref().to_str().ok_or("Bad path provided")?;
        debug!("Writing file: {}", path);
        opencv::imgcodecs::imwrite(path, &self.inner, &opencv::core::Vector::default())?;
        Ok(())
    }

    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(Self::from(opencv::imgcodecs::imread(
            path.to_str().ok_or("Image path isn't valid unicode")?,
            1,
        )?))
    }

    pub fn into_inner(self) -> Mat {
        self.inner
    }
}

impl From<Mat> for Picture {
    fn from(inner: Mat) -> Self {
        Self {
            inner: inner.clone(),
        }
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
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        Ok(Self {
            camera_handle,
            _grabber_thread_join,
        })
    }

    pub fn new_default() -> Result<Self, Box<dyn Error>> {
        Self::new_local(0)
    }

    pub fn new_local(index: i32) -> Result<Self, Box<dyn Error>> {
        let cam = videoio::VideoCapture::new(index, videoio::CAP_ANY)?; // 0 is the default camera
        Self::new_from_video_capture(cam)
    }

    pub fn new_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let cam = videoio::VideoCapture::from_file(path, videoio::CAP_ANY)?; // 0 is the default camera
        Self::new_from_video_capture(cam)
    }

    pub fn capture(&mut self) -> Result<Picture, Box<dyn Error>> {
        let mut frame = Mat::default();
        let mut camera_handle = self.camera_handle.lock().unwrap();
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
        // hack to get the window to close, see https://stackoverflow.com/a/50710994
        highgui::wait_key(1).unwrap();
    }
}

pub fn preprocess_diff(
    base_picture: &Picture,
    led_picture: &Picture,
    output_dir: Option<&Path>,
) -> Result<Mat, Box<dyn Error>> {
    let mut base_gray = Mat::default();
    imgproc::cvt_color(
        &base_picture.inner,
        &mut base_gray,
        imgproc::COLOR_BGR2GRAY,
        0,
    )?;
    if let Some(output_dir) = &output_dir {
        Picture::from(base_gray.clone())
            .save_to_file(output_dir.with_file_name("1_base_grayscale.jpg"))?;
    }
    let mut led_gray = Mat::default();
    imgproc::cvt_color(
        &led_picture.inner,
        &mut led_gray,
        imgproc::COLOR_BGR2GRAY,
        0,
    )?;
    if let Some(output_dir) = &output_dir {
        Picture::from(led_gray.clone())
            .save_to_file(output_dir.with_file_name("2_led_grayscale.jpg"))?;
    }
    let mut diff = Mat::default();
    core::absdiff(&base_gray, &led_gray, &mut diff)?;
    if let Some(output_dir) = &output_dir {
        Picture::from(diff.clone()).save_to_file(output_dir.with_file_name("3_diff.jpg"))?;
    }

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
    let eroded = Picture::from(eroded);
    if let Some(output_dir) = &output_dir {
        eroded.save_to_file(output_dir.with_file_name("4_eroded.jpg"))?;
    }

    Ok(eroded.inner)
}

pub fn find_light_from_diff(
    base_picture: &Picture,
    led_picture: &Picture,
) -> Result<WithConfidence<(usize, usize)>, Box<dyn Error>> {
    find_light_from_diff_with_output(base_picture, led_picture, None)
}

pub fn find_light_from_diff_with_output(
    base_picture: &Picture,
    led_picture: &Picture,
    output_dir: Option<&Path>,
) -> Result<WithConfidence<(usize, usize)>, Box<dyn Error>> {
    let eroded = preprocess_diff(base_picture, led_picture, output_dir)?;

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
    let result = WithConfidence::<(usize, usize)> {
        inner: (max_loc.x as usize, max_loc.y as usize),
        confidence: max_val / 255.0,
    };
    if let Some(output_dir) = &output_dir {
        // eroded image is in grayscale but we want to mark it in color, so we convert it
        let mut eroded_color = Mat::default();
        imgproc::cvt_color(&eroded, &mut eroded_color, COLOR_GRAY2RGB, 0)?;
        let mut pic = Picture::from(eroded_color);
        pic.mark(&result)?;
        pic.save_to_file(output_dir.with_file_name("5_eroded_marked.jpg"))?;
    }

    Ok(result)
}

pub fn find_light_from_bit_images(
    index: usize,
    positive_images: &[Mat],
    negative_images: &[Mat],
) -> WithConfidence<(usize, usize)> {
    let base = Mat::ones(positive_images[0].rows(), positive_images[0].cols(), CV_32F)
        .unwrap()
        .to_mat()
        .unwrap();

    let multiplied = positive_images
        .iter()
        .zip(negative_images)
        .enumerate()
        .map(|(i, (positive, negative))| {
            if 1 >> i & index != 0 {
                positive
            } else {
                negative
            }
        })
        .fold(base, |acc, elem| {
            let mut mult = Mat::default();
            opencv::core::multiply(&acc, elem, &mut mult, 1.0, -1).unwrap();
            mult
            // let mut sqrt = Mat::default();
            // opencv::core::sqrt(&mult, &mut sqrt).unwrap();
            // sqrt
        });

    let mut max_loc = core::Point::default();
    let mut max_val: f64 = 0.0;
    opencv::core::min_max_loc(
        &multiplied,
        None,
        Some(&mut max_val),
        None,
        Some(&mut max_loc),
        &Mat::default(),
    )
    .unwrap();
    if max_loc.x < 0 || max_loc.y < 0 {
        // OpenCV might return (-1,-1) if it can't find anything
        return WithConfidence {
            inner: (0, 0),
            confidence: f64::NEG_INFINITY,
        };
    }
    WithConfidence {
        inner: (max_loc.x as usize, max_loc.y as usize),
        confidence: max_val,
    }
}
