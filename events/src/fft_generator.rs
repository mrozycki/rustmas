use animation_api::event::Event;
use anyhow::anyhow;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleFormat,
};
use itertools::Itertools;
use log::error;
use rustfft::{num_complex::Complex32, FftPlanner};
use tokio::sync::mpsc;

use std::{
    collections::VecDeque,
    error::Error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

const FFT_SIZE: usize = 1024;
const SAMPLE_RATE: usize = 48000;

pub struct FftEventGenerator {
    keep_running: Arc<AtomicBool>,
    _input_stream_handle: std::thread::JoinHandle<()>,
    _fft_loop_handle: std::thread::JoinHandle<()>,
}

impl FftEventGenerator {
    pub fn new(fps: f64, channel: mpsc::Sender<Event>) -> Result<Self, Box<dyn Error>> {
        let keep_running = Arc::new(AtomicBool::new(true));
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No default input device found"))?;
        let config = device.default_input_config()?;
        let buffer: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(vec![0.0; FFT_SIZE].into()));

        let input_stream_handle = {
            let buffer_push = {
                let buffer = buffer.clone();
                move |data: &[f32]| {
                    let mut buf = buffer.lock().unwrap();
                    for &sample in data {
                        buf.push_front(sample);
                        buf.pop_back();
                    }
                }
            };

            let err_fn = |err| {
                error!("an error occurred on stream: {}", err);
            };

            // Workaround for stream not being Send; we don't need it to be sent
            // between threads, we just need it to keep running in the background
            // until the generator is dropped.
            let keep_running = keep_running.clone();
            std::thread::spawn(move || {
                let stream = match config.sample_format() {
                    SampleFormat::I8 => device.build_input_stream(
                        &config.into(),
                        move |data: &[i8], _: &cpal::InputCallbackInfo| {
                            buffer_push(&data.iter().copied().map(f32::from_sample).collect_vec())
                        },
                        err_fn,
                        None,
                    ),
                    SampleFormat::I16 => device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            buffer_push(&data.iter().copied().map(f32::from_sample).collect_vec())
                        },
                        err_fn,
                        None,
                    ),
                    SampleFormat::I32 => device.build_input_stream(
                        &config.into(),
                        move |data: &[i32], _: &cpal::InputCallbackInfo| {
                            buffer_push(&data.iter().copied().map(f32::from_sample).collect_vec())
                        },
                        err_fn,
                        None,
                    ),
                    SampleFormat::F32 => device.build_input_stream(
                        &config.into(),
                        move |data, _: &cpal::InputCallbackInfo| buffer_push(data),
                        err_fn,
                        None,
                    ),
                    sample_format => {
                        error!("Unsupported sample format '{sample_format}'");
                        return;
                    }
                }
                .map_err(|e| anyhow!(e))
                .and_then(|stream| {
                    stream.play()?;
                    Ok(stream)
                });

                if let Err(e) = stream {
                    error!(
                        "Failed to build audio input stream, audio animations will not work: {}",
                        e
                    );
                    return;
                };

                while keep_running.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(1000));
                }
            })
        };

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        let bands = (2..20).map(|i| (i * 1000, (i + 1) * 1000)).collect_vec();

        let event_generator_thread = thread::spawn(move || {
            while !channel.is_closed() {
                let mut spectrum = {
                    let buf = buffer.lock().unwrap();
                    buf.iter().map(|x| Complex32::new(*x, 0.0)).collect_vec()
                };
                fft.process(spectrum.as_mut_slice());
                let bands = bands
                    .iter()
                    .map(|(low, high)| {
                        let low_bin =
                            (*low as f32 / SAMPLE_RATE as f32 * FFT_SIZE as f32).round() as usize;
                        let high_bin =
                            (*high as f32 / SAMPLE_RATE as f32 * FFT_SIZE as f32).round() as usize;

                        spectrum[low_bin..high_bin]
                            .iter()
                            .map(|c| c.norm())
                            .sum::<f32>()
                            / (high_bin - low_bin) as f32
                    })
                    .collect_vec();
                let _ = channel.blocking_send(Event::FftEvent { bands });
                std::thread::sleep(Duration::from_secs_f64(1.0 / fps));
            }
        });

        Ok(Self {
            keep_running,
            _input_stream_handle: input_stream_handle,
            _fft_loop_handle: event_generator_thread,
        })
    }
}

impl Drop for FftEventGenerator {
    fn drop(&mut self) {
        self.keep_running.store(false, Ordering::Relaxed);
    }
}
