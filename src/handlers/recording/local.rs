use super::handler::RecordingHandler;
use crate::error::{Error, Result};
use bytes::Bytes;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Device;
use hound::{WavSpec, WavWriter};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: u32 = 16000;
const CHANNELS: u16 = 1;

pub struct LocalRecorder {
    stream: cpal::Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl LocalRecorder {
    pub fn new(device_name: impl Into<String>) -> Result<Self> {
        let device_name = device_name.into();
        let host = cpal::default_host();

        let device = host
            .input_devices()?
            .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
            .ok_or(Error::AudioInputDeviceNotFound(device_name))?;

        Self::set_up_recorder(device)
    }

    pub fn new_default() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(Error::NoDefaultAudioInputDevice)?;

        Self::set_up_recorder(device)
    }

    fn set_up_recorder(device: Device) -> Result<Self> {
        let config = cpal::StreamConfig {
            channels: CHANNELS,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = buffer.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut buffer) = buffer_clone.lock() {
                    buffer.extend_from_slice(data);
                }
            },
            move |err| {
                eprintln!("An error occurred on stream: {}", err);
            },
            None,
        )?;

        Ok(Self { stream, buffer })
    }
}

impl RecordingHandler for LocalRecorder {
    fn start(&self) -> Result<()> {
        self.buffer
            .lock()
            .map_err(|_| Error::Lock("recording buffer".to_owned()))?
            .clear();
        self.stream.play()?;
        Ok(())
    }

    fn stop(&self) -> Result<Bytes> {
        self.stream.pause()?;
        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let recorded_data = self
            .buffer
            .lock()
            .map_err(|_| Error::Lock("recording buffer".to_owned()))?
            .clone();
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut cursor, spec)?;
            for &sample in &recorded_data {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        }

        Ok(Bytes::from(cursor.into_inner()))
    }
}
