use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{error::Error, time::Duration};

use image::buffer::ConvertBuffer;
use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage};
use imageproc::drawing::draw_hollow_circle_mut;
use imageproc::integral_image::ArrayData;
use imageproc::map::map_colors2;
use imageproc::morphology::{erode, grayscale_erode, Mask};
use imageproc::template_matching::find_extremes;
use log::debug;

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
    inner: image::RgbImage,
}

impl Picture {
    pub fn mark(&mut self, coords: &WithConfidence<(usize, usize)>) -> Result<(), Box<dyn Error>> {
        let color = if coords.confident() {
            Rgb([0u8, 255u8, 0u8]) // green
        } else {
            Rgb([255u8, 0u8, 0u8]) // red
        };
        draw_hollow_circle_mut(
            &mut self.inner,
            (coords.inner.0 as i32, coords.inner.1 as i32),
            20,
            color,
        );
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref().to_str().ok_or("Bad path provided")?;
        debug!("Writing file: {}", path);
        self.inner.save(path)?;
        Ok(())
    }

    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(image::ImageReader::open(path)?.decode()?.into_rgb8().into())
    }
}

impl From<image::RgbImage> for Picture {
    fn from(inner: image::RgbImage) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

pub struct Camera {
    // camera_handle: Arc<Mutex<videoio::VideoCapture>>,
    // _grabber_thread_join: JoinHandle<()>,
}

impl Camera {
    // pub fn new_from_video_capture(mut capture: VideoCapture) -> Result<Self, Box<dyn Error>> {
    //     capture.set(videoio::CAP_PROP_BUFFERSIZE, 1.0)?;

    //     if !videoio::VideoCapture::is_opened(&capture)? {
    //         return Err(Box::new(CameraError::InitializeError));
    //     }
    //     let camera_handle = Arc::new(Mutex::new(capture));
    //     let camera_handle_clone = camera_handle.clone();
    //     let _grabber_thread_join = tokio::spawn(async move {
    //         loop {
    //             let _ = camera_handle_clone.lock().unwrap().grab();
    //             tokio::time::sleep(Duration::from_millis(10)).await;
    //         }
    //     });
    //     Ok(Self {
    //         camera_handle,
    //         _grabber_thread_join,
    //     })
    // }

    pub fn new_default() -> Result<Self, Box<dyn Error>> {
        // Self::new_local(0)
        Ok(Self {})
    }

    pub fn new_local(index: i32) -> Result<Self, Box<dyn Error>> {
        // let cam = videoio::VideoCapture::new(index, videoio::CAP_ANY)?; // 0 is the default camera
        // Self::new_from_video_capture(cam)
        Ok(Self {})
    }

    pub fn new_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        // let cam = videoio::VideoCapture::from_file(path, videoio::CAP_ANY)?; // 0 is the default camera
        // Self::new_from_video_capture(cam)
        Ok(Self {})
    }

    pub fn capture(&mut self) -> Result<Picture, Box<dyn Error>> {
        // let mut frame = Mat::default();
        // let mut camera_handle = self.camera_handle.lock().unwrap();
        // camera_handle.read(&mut frame)?;

        // if frame.size()?.width > 0 {
        //     Ok(frame.into())
        // } else {
        //     Err(Box::new(CameraError::CaptureError))
        // }
        todo!();
    }
}

// impl Drop for Camera {
//     fn drop(&mut self) {
//         self.camera_handle.lock().unwrap().release().unwrap();
//     }
// }

pub struct Display {
    window_name: String,
}

impl Display {
    pub fn new(window_name: &str) -> Result<Self, Box<dyn Error>> {
        // highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE)?;
        // highgui::set_window_property(window_name, highgui::WND_PROP_TOPMOST, 1.0)?;

        Ok(Self {
            window_name: window_name.to_owned(),
        })
    }

    pub fn show(&self, picture: &Picture) -> Result<(), Box<dyn Error>> {
        // highgui::imshow(&self.window_name, &picture.inner)?;
        Ok(())
    }

    pub fn wait_for(&self, duration: Duration) -> Result<i32, Box<dyn Error>> {
        // Ok(highgui::wait_key(duration.as_millis() as i32)?)
        Ok(0)
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        // highgui::destroy_window(&self.window_name).unwrap();
        // hack to get the window to close, see https://stackoverflow.com/a/50710994
        // highgui::wait_key(1).unwrap();
    }
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
    output_dir: Option<PathBuf>,
) -> Result<WithConfidence<(usize, usize)>, Box<dyn Error>> {
    let base_gray: GrayImage = base_picture.inner.convert();
    if let Some(output_dir) = &output_dir {
        Picture::from(base_gray.clone().convert())
            .save_to_file(output_dir.with_file_name("1_base_grayscale.jpg"))?;
    }

    let led_gray: GrayImage = led_picture.inner.convert();
    if let Some(output_dir) = &output_dir {
        Picture::from(led_gray.clone().convert())
            .save_to_file(output_dir.with_file_name("2_led_grayscale.jpg"))?;
    }

    let diff: GrayImage = map_colors2(&base_gray, &led_gray, |a, b| Luma([a[0].abs_diff(b[0])]));

    if let Some(output_dir) = &output_dir {
        Picture::from(diff.clone().convert())
            .save_to_file(output_dir.with_file_name("3_diff.jpg"))?;
    }

    // erode to remove the noise
    let eroded = grayscale_erode(&diff, &Mask::diamond(3u8));

    if let Some(output_dir) = &output_dir {
        Picture::from(eroded.clone().convert())
            .save_to_file(output_dir.with_file_name("4_eroded.jpg"))?;
    }

    let extremes = find_extremes(&eroded);

    let max_loc = extremes.max_value_location;
    let max_val = extremes.max_value;

    let result = WithConfidence::<(usize, usize)> {
        inner: (max_loc.0 as usize, max_loc.1 as usize),
        confidence: max_val as f64 / 255.0,
    };
    if let Some(output_dir) = &output_dir {
        let mut pic = Picture::from(eroded.convert());
        pic.mark(&result)?;
        pic.save_to_file(output_dir.with_file_name("5_eroded_marked.jpg"))?;
    }

    Ok(result)
}
