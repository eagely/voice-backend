use super::recording_service::RecordingService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{default_host, BufferSize, InputCallbackInfo, SampleRate, StreamConfig};
use hound::{SampleFormat, WavSpec, WavWriter};
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc::{self, Receiver, Sender};

const SAMPLE_RATE: u32 = 16000;
const CHANNELS: u16 = 1;
const RB_CAPACITY: usize = SAMPLE_RATE as usize * 600;

struct CpalRunner;

impl CpalRunner {
    fn run(
        device_name: String,
        buffer: Arc<Mutex<VecDeque<f32>>>,
        total_written: Arc<AtomicUsize>,
        error_tx: Sender<Error>,
    ) -> Result<()> {
        let host = default_host();
        let device = host
            .input_devices()?
            .find(|d| d.name().ok().filter(|n| n == &device_name).is_some())
            .ok_or(Error::AudioInputDeviceNotFound(device_name.clone()))?;
        let config = StreamConfig {
            channels: CHANNELS,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: BufferSize::Default,
        };

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _info: &InputCallbackInfo| {
                if let Ok(mut buf) = buffer.lock() {
                    for &sample in data {
                        if buf.len() == RB_CAPACITY {
                            buf.pop_front();
                        }
                        buf.push_back(sample);
                        total_written.fetch_add(1, Ordering::Relaxed);
                    }
                }
            },
            move |err| {
                let _ = error_tx.try_send(err.into());
            },
            None,
        )?;

        stream.play()?;

        loop {
            thread::park();
        }
    }
}

pub struct LocalRecorder {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    total_written: Arc<AtomicUsize>,
    start_index: Arc<Mutex<Option<usize>>>,
    error_rx: Mutex<Receiver<Error>>,
}

impl LocalRecorder {
    pub fn new(device_name: impl Into<String>) -> Result<Self> {
        let device_name = device_name.into();
        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(RB_CAPACITY)));
        let total_written = Arc::new(AtomicUsize::new(0));
        let start_index = Arc::new(Mutex::new(None));

        let (error_tx, error_rx) = mpsc::channel(1);

        let buffer_clone = Arc::clone(&buffer);
        let total_written_clone = Arc::clone(&total_written);
        let device_name_clone = device_name.clone();
        let error_tx_clone = error_tx.clone();
        thread::spawn(move || {
            if let Err(e) = CpalRunner::run(
                device_name_clone,
                buffer_clone,
                total_written_clone,
                error_tx_clone,
            ) {
                let _ = error_tx.try_send(e);
            }
        });

        Ok(LocalRecorder {
            buffer,
            total_written,
            start_index,
            error_rx: Mutex::new(error_rx),
        })
    }

    fn check_error(&self) -> Option<Error> {
        if let Ok(mut rx) = self.error_rx.lock() {
            if let Ok(err) = rx.try_recv() {
                return Some(err);
            }
        }
        None
    }
}

#[async_trait]
impl RecordingService for LocalRecorder {
    async fn start(&self) -> Result<()> {
        if let Some(err) = self.check_error() {
            return Err(err);
        }
        let current = self.total_written.load(Ordering::Relaxed);
        let mut lock = self
            .start_index
            .lock()
            .map_err(|_| Error::Lock("start_index".into()))?;
        *lock = Some(current);
        Ok(())
    }

    async fn stop(&self) -> Result<Bytes> {
        if let Some(err) = self.check_error() {
            return Err(err);
        }
        let current = self.total_written.load(Ordering::Relaxed);
        let start = {
            let mut lock = self
                .start_index
                .lock()
                .map_err(|_| Error::Lock("start_index".into()))?;
            lock.take().unwrap_or(current)
        };

        let snapshot = {
            let lock = self
                .buffer
                .lock()
                .map_err(|_| Error::Lock("buffer".into()))?;
            lock.clone()
        };

        let absolute_start_index = current.saturating_sub(snapshot.len());
        let effective_start = if start < absolute_start_index {
            absolute_start_index
        } else {
            start
        };

        let sample_count = current.saturating_sub(effective_start);
        let offset = effective_start.saturating_sub(absolute_start_index);
        let available_samples = snapshot.len().saturating_sub(offset);
        let count = sample_count.min(available_samples);

        let recorded: Vec<f32> = snapshot.into_iter().skip(offset).take(count).collect();

        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut cursor, spec)?;
            for sample in recorded {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        }
        Ok(Bytes::from(cursor.into_inner()))
    }
}
