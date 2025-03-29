/*
 * Continuously captures audio into a ring buffer.
 * When recording starts, it marks the current position in the buffer.
 * When stopped, it extracts samples between the start and stop positions to create a WAV file.
 */
use super::recording_service::RecordingService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{default_host, BufferSize, InputCallbackInfo, SampleRate, StreamConfig};
use hound::{SampleFormat, WavSpec, WavWriter};
use log::info;
use porcupine::PorcupineBuilder;
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc::{self, Receiver, Sender};

const SAMPLE_RATE: u32 = 16000;
const CHANNELS: u16 = 1;
const RB_CAPACITY: usize = SAMPLE_RATE as usize * 600; // 10 minute buffer

pub struct LocalRecorder {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    total_samples_captured: Arc<AtomicUsize>,
    start_index: Arc<Mutex<Option<usize>>>,
    error_rx: Mutex<Receiver<Error>>,
    is_recording: Arc<AtomicBool>,
}

impl LocalRecorder {
    pub fn new(
        device_name: impl Into<String>,
        access_key: impl Into<String>,
        keyword_path: impl Into<String>,
        wake_word_enabled: bool,
        sensitivity: f32,
    ) -> Result<Self> {
        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(RB_CAPACITY)));
        let total_samples_captured = Arc::new(AtomicUsize::new(0));
        let start_index = Arc::new(Mutex::new(None));
        let is_recording = Arc::new(AtomicBool::new(false));
        let wake_word_enabled = Arc::new(AtomicBool::new(wake_word_enabled));

        let (error_tx, error_rx) = mpsc::channel(1);

        Self::spawn_audio_thread(
            device_name.into(),
            buffer.clone(),
            total_samples_captured.clone(),
            error_tx,
            is_recording.clone(),
            start_index.clone(),
            wake_word_enabled.clone(),
            access_key.into(),
            keyword_path.into(),
            sensitivity,
        )?;

        Ok(Self {
            buffer,
            total_samples_captured,
            start_index,
            error_rx: Mutex::new(error_rx),
            is_recording,
        })
    }

    fn spawn_audio_thread(
        device_name: String,
        buffer: Arc<Mutex<VecDeque<f32>>>,
        total_samples_captured: Arc<AtomicUsize>,
        error_tx: Sender<Error>,
        is_recording: Arc<AtomicBool>,
        start_index: Arc<Mutex<Option<usize>>>,
        wake_word_enabled: Arc<AtomicBool>,
        access_key: String,
        keyword_path: String,
        sensitivity: f32,
    ) -> Result<()> {
        thread::spawn(move || {
            if let Err(e) = Self::run_audio_capture(
                device_name,
                buffer,
                total_samples_captured,
                error_tx.clone(),
                is_recording,
                start_index,
                wake_word_enabled,
                access_key,
                keyword_path,
                sensitivity,
            ) {
                let _ = error_tx.try_send(e);
            }
        });

        Ok(())
    }

    fn run_audio_capture(
        device_name: String,
        buffer: Arc<Mutex<VecDeque<f32>>>,
        total_samples_captured: Arc<AtomicUsize>,
        error_tx: Sender<Error>,
        is_recording: Arc<AtomicBool>,
        start_index: Arc<Mutex<Option<usize>>>,
        wake_word_enabled: Arc<AtomicBool>,
        access_key: String,
        keyword_path: String,
        sensitivity: f32,
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

        let porcupine = PorcupineBuilder::new_with_keyword_paths(access_key, &[keyword_path])
            .sensitivities(&[sensitivity])
            .init()?;

        let frame_length = porcupine.frame_length() as usize;
        let mut frame_buffer = Vec::with_capacity(frame_length);

        let data_error_tx = error_tx.clone();
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _info: &InputCallbackInfo| {
                Self::process_audio_data(
                    data,
                    &buffer,
                    &total_samples_captured,
                    &mut frame_buffer,
                    frame_length,
                    &porcupine,
                    &wake_word_enabled,
                    &is_recording,
                    &start_index,
                    &data_error_tx,
                );
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

    fn process_audio_data(
        data: &[f32],
        buffer: &Arc<Mutex<VecDeque<f32>>>,
        total_samples_captured: &Arc<AtomicUsize>,
        frame_buffer: &mut Vec<i16>,
        frame_length: usize,
        porcupine: &porcupine::Porcupine,
        wake_word_enabled: &Arc<AtomicBool>,
        is_recording: &Arc<AtomicBool>,
        start_index: &Arc<Mutex<Option<usize>>>,
        error_tx: &Sender<Error>,
    ) {
        if let Ok(mut buf) = buffer.lock() {
            for &sample in data {
                if buf.len() == RB_CAPACITY {
                    buf.pop_front();
                }
                buf.push_back(sample);
                total_samples_captured.fetch_add(1, Ordering::Relaxed);

                let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                frame_buffer.push(sample_i16);

                if frame_buffer.len() >= frame_length {
                    Self::process_wake_word_frame(
                        frame_buffer,
                        porcupine,
                        wake_word_enabled,
                        is_recording,
                        total_samples_captured,
                        start_index,
                        error_tx,
                    );
                }
            }
        }
    }

    fn process_wake_word_frame(
        frame_buffer: &mut Vec<i16>,
        porcupine: &porcupine::Porcupine,
        wake_word_enabled: &Arc<AtomicBool>,
        is_recording: &Arc<AtomicBool>,
        total_samples_captured: &Arc<AtomicUsize>,
        start_index: &Arc<Mutex<Option<usize>>>,
        error_tx: &Sender<Error>,
    ) {
        match porcupine.process(frame_buffer) {
            Ok(keyword_index) => {
                if keyword_index >= 0 {
                    info!("Keyword triggered");

                    if wake_word_enabled.load(Ordering::Relaxed)
                        && !is_recording.load(Ordering::Relaxed)
                    {
                        is_recording.store(true, Ordering::Relaxed);
                        let current = total_samples_captured.load(Ordering::Relaxed);
                        if let Ok(mut lock) = start_index.lock() {
                            *lock = Some(current);
                        }
                    }
                }
            }
            Err(e) => {
                let _ = error_tx.try_send(e.into());
            }
        }
        frame_buffer.clear();
    }

    fn check_error(&self) -> Option<Error> {
        if let Ok(mut rx) = self.error_rx.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }

    fn create_wav_from_samples(samples: &[f32]) -> Result<Bytes> {
        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut cursor, spec)?;
            for &sample in samples {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        }

        Ok(Bytes::from(cursor.into_inner()))
    }
}

#[async_trait]
impl RecordingService for LocalRecorder {
    async fn start(&self) -> Result<()> {
        if let Some(err) = self.check_error() {
            return Err(err);
        }

        self.is_recording.store(true, Ordering::Relaxed);
        let current = self.total_samples_captured.load(Ordering::Relaxed);

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

        self.is_recording.store(false, Ordering::Relaxed);

        let current = self.total_samples_captured.load(Ordering::Relaxed);
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

        // Calculate appropriate indices for the recorded segment
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

        Self::create_wav_from_samples(&recorded)
    }
}
