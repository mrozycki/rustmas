use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::{error::Error, time::Duration};

use image::buffer::ConvertBuffer;
use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage, Rgba, RgbaImage};
use imageproc::drawing::draw_hollow_circle_mut;
use imageproc::integral_image::ArrayData;
use imageproc::map::map_colors2;
use imageproc::morphology::{erode, grayscale_erode, Mask};
use imageproc::template_matching::find_extremes;
use itertools::Itertools;
use log::{debug, info, warn};
use winit::window::WindowAttributes;

use std::thread::JoinHandle;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{DeviceEvent, ElementState, Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};

use crate::capture::WithConfidence;

#[derive(Debug)]
pub enum CameraError {
    InitializeError,
    DeviceNotFoundError,
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
    inner: image::RgbaImage,
}

impl Picture {
    pub fn mark(&mut self, coords: &WithConfidence<(usize, usize)>) -> Result<(), Box<dyn Error>> {
        let color = if coords.confident() {
            Rgba([1u8, 255u8, 0u8, 255]) // green
        } else {
            Rgba([255u8, 0u8, 0u8, 255]) // red
        };
        // currently there is no way to draw thicker lines, so we draw a couple of lines
        // see https://github.com/image-rs/imageproc/issues/513
        let thickness = 2;
        let radius = 20;
        for i in 0..thickness {
            draw_hollow_circle_mut(
                &mut self.inner,
                (coords.inner.0 as i32, coords.inner.1 as i32),
                radius + i,
                color,
            );
        }
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref().to_str().ok_or("Bad path provided")?;
        debug!("Writing file: {}", path);
        let converted: RgbImage = self.inner.convert();
        converted.save(path)?;
        Ok(())
    }

    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(image::ImageReader::open(path)?
            .decode()?
            .into_rgba8()
            .into())
    }
}

impl From<image::RgbaImage> for Picture {
    fn from(inner: image::RgbaImage) -> Self {
        Self {
            inner: inner.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    camera: kamera::Camera,
}

impl Camera {
    pub fn new_default() -> Result<Self, Box<dyn Error>> {
        Self::new_local(0)
    }

    pub fn new_local(index: usize) -> Result<Self, Box<dyn Error>> {
        let mut devices = kamera::Device::list_all_devices();
        if index >= devices.len() {
            warn!("Camera index out of range!");
            return Err(Box::new(CameraError::DeviceNotFoundError));
        }
        let device = devices.remove(index);
        info!("Using camera: {}", device.name());
        let camera = kamera::Camera::new_from_device(device);
        camera.start();

        Ok(Self { camera })
    }

    pub fn list_devices() -> Vec<String> {
        kamera::Device::list_all_devices()
            .into_iter()
            .map(|dev| dev.name())
            .collect()
    }

    pub fn capture(&mut self) -> Result<Picture, Box<dyn Error>> {
        let Some(frame) = self.camera.wait_for_frame() else {
            warn!("wait for frame failed");
            return Err(Box::new(CameraError::CaptureError));
        };
        let (width, height) = frame.size_u32();
        info!(
            "Converting: {width}, {height}, {:?}",
            frame.data().data_u8().as_ptr()
        );
        let Some(img) = RgbaImage::from_raw(
            width,
            height,
            frame
                .data()
                .data_u32()
                .iter()
                .flat_map(|p| {
                    let bytes = p.to_le_bytes();
                    [bytes[2], bytes[1], bytes[0], bytes[3]]
                })
                .collect_vec(),
        ) else {
            warn!("conversion failed!");
            return Err(Box::new(CameraError::CaptureError));
        };
        Ok(Picture::from(img))
    }
}

pub struct Display {
    // window: Window,
    loop_stopper: Arc<AtomicBool>,
    next_frame: Arc<RwLock<Option<Picture>>>,
    _grabber_thread_join: JoinHandle<()>,
}

impl Display {
    #[allow(deprecated)]
    pub fn preview(
        mut camera: Camera,
        loop_stopper: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error>> {
        let event_loop = EventLoop::new().unwrap();
        // let (w, h) = camera.capture().unwrap().inner.dimensions();
        // let window_attr = WindowAttributes::default().with_inner_size(PhysicalSize::new(w, h));
        let window = event_loop.create_window(Default::default()).unwrap();
        let context = softbuffer::Context::new(&window).unwrap();
        let mut surface = softbuffer::Surface::new(&context, &window).unwrap();
        // surface
        //     .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
        //     .unwrap();
        event_loop
            .run(|event, event_loop| {
                if loop_stopper.load(Ordering::Relaxed) {
                    event_loop.exit();
                    return;
                }

                match event {
                    Event::WindowEvent {
                        window_id,
                        event: WindowEvent::RedrawRequested,
                    } if window_id == window.id() => {
                        // info!("redraw requested");
                        if let Ok(frame) = camera.capture() {
                            info!("got frame");
                            let (w, h) = frame.inner.dimensions();
                            let window_size = window.inner_size();

                            if window_size.width != w || window_size.height != h {
                                let _ = window.request_inner_size(PhysicalSize::new(w, h));
                                info!("bad size");
                                return;
                            } else {
                                let mut buffer = surface.buffer_mut().unwrap();

                                frame.inner.pixels().zip(buffer.iter_mut()).for_each(
                                    |(from, to)| {
                                        *to = u32::from_le_bytes([
                                            from.0[2], from.0[1], from.0[0], 255,
                                        ]);
                                    },
                                );
                                buffer.present().unwrap();
                            }
                        }
                        window.request_redraw();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == window.id() => {
                        event_loop.exit();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::Resized(physical_size),
                        window_id,
                    } if window_id == window.id() => {
                        info!("resized: {:?}", event);
                        surface
                            .resize(
                                NonZeroU32::new(physical_size.width).unwrap(),
                                NonZeroU32::new(physical_size.height).unwrap(),
                            )
                            .unwrap();

                        window.request_redraw();
                    }
                    _ => {}
                }
            })
            .unwrap();

        Ok(())
    }

    pub fn show(&self, picture: Picture) -> Result<(), Box<dyn Error>> {
        // highgui::imshow(&self.window_name, &picture.inner)?;
        *self.next_frame.write().unwrap() = Some(picture);
        Ok(())
    }

    pub fn wait_for(&self, duration: Duration) -> Result<i32, Box<dyn Error>> {
        // Ok(highgui::wait_key(duration.as_millis() as i32)?)
        Ok(0)
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        self.loop_stopper.store(true, Ordering::Relaxed);
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
