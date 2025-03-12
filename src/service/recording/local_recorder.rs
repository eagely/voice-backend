use super::recording_service::RecordingService;
use crate::error::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{default_host, BufferSize, Device, InputCallbackInfo, SampleRate, Stream, StreamConfig};
use hound::{WavSpec, WavWriter};
use ringbuf::traits::{Consumer, RingBuffer};
use ringbuf::HeapRb;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 16000;
const CHANNELS: u16 = 1;
const RB_CAPACITY: usize = SAMPLE_RATE as usize * 600;

pub struct UnsafeSendSync<T>(pub T);
unsafe impl<T> Sync for UnsafeSendSync<T> {}
unsafe impl<T> Send for UnsafeSendSync<T> {}

pub struct LocalRecorder {
    stream: UnsafeSendSync<Stream>,
    buffer: Arc<Mutex<HeapRb<f32>>>,
    total_written: Arc<AtomicUsize>,
    start_index: Arc<Mutex<Option<usize>>>,
}

impl LocalRecorder {
    pub fn new(device_name: impl Into<String>) -> Result<Self> {
        let device_name = device_name.into();
        let host = default_host();

        let device = host
            .input_devices()?
            .find(|d| d.name().is_ok_and(|n| n == device_name))
            .ok_or(Error::AudioInputDeviceNotFound(device_name))?;

        Self::set_up_recorder(device)
    }

    fn set_up_recorder(device: Device) -> Result<Self> {
        let config = StreamConfig {
            channels: CHANNELS,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: BufferSize::Default,
        };

        let rb = Arc::new(Mutex::new(HeapRb::<f32>::new(RB_CAPACITY)));
        let total_written = Arc::new(AtomicUsize::new(0));
        let start_index = Arc::new(Mutex::new(None));

        let rb_clone = rb.clone();
        let total_written_clone = total_written.clone();
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &InputCallbackInfo| {
                if let Ok(mut rb) = rb_clone.lock() {
                    for &sample in data {
                        rb.push_overwrite(sample);
                        total_written_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
            },
            move |err| {
                eprintln!("An error occurred on stream: {}", err);
            },
            None,
        )?;

        let stream = UnsafeSendSync(stream);

        Ok(Self {
            stream,
            buffer: rb,
            total_written,
            start_index,
        })
    }
}

#[async_trait]
impl RecordingService for LocalRecorder {
    async fn start(&self) -> Result<()> {
        let current = self.total_written.load(Ordering::Relaxed);
        let mut start = self
            .start_index
            .lock()
            .map_err(|_| Error::Lock("start_index".to_string()))?;
        *start = Some(current);

        self.stream.0.play()?;
        Ok(())
    }

    async fn stop(&self) -> Result<Bytes> {
        self.stream.0.pause()?;

        let current = self.total_written.load(Ordering::Relaxed);
        let start = {
            let mut start_lock = self
                .start_index
                .lock()
                .map_err(|_| Error::Lock("start_index".to_string()))?;
            start_lock.take().unwrap_or(current)
        };

        let effective_start = if start < current.saturating_sub(RB_CAPACITY) {
            current.saturating_sub(RB_CAPACITY)
        } else {
            start
        };
        let sample_count = current.saturating_sub(effective_start);

        let rb_lock = self
            .buffer
            .lock()
            .map_err(|_| Error::Lock("buffer".to_string()))?;
        let (first, second) = rb_lock.as_slices();
        let mut snapshot = Vec::with_capacity(first.len() + second.len());
        snapshot.extend_from_slice(first);
        snapshot.extend_from_slice(second);
        drop(rb_lock);

        let offset = if current > RB_CAPACITY {
            effective_start.saturating_sub(current - RB_CAPACITY)
        } else {
            effective_start
        };

        let sample_count = sample_count.min(snapshot.len().saturating_sub(offset));
        let recorded = snapshot[offset..offset + sample_count].to_vec();

        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
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
