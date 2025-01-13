use std::{io::Cursor, sync::Arc};

use super::handler::TranscriptionHandler;
use crate::error::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use hound::WavReader;
use whisper_rs::{FullParams, WhisperContext, WhisperContextParameters};

pub struct LocalWhisperTranscriber {
    pub context: Arc<WhisperContext>,
}

impl LocalWhisperTranscriber {
    pub fn new(model: impl Into<String>) -> Result<Self> {
        let mut params = WhisperContextParameters::default();
        params.use_gpu = true;
        let context = Arc::new(WhisperContext::new_with_params(&model.into(), params)?);
        Ok(Self { context })
    }
}

#[async_trait]
impl TranscriptionHandler for LocalWhisperTranscriber {
    async fn transcribe(&self, audio: &Bytes) -> Result<String> {
        let cursor = Cursor::new(audio);
        let mut reader = WavReader::new(cursor)?;

        let spec = reader.spec();
        if spec.channels != 1 || spec.sample_rate != 16000 {
            return Err(Error::AudioCodec("Wav must be 16".to_owned()));
        }

        let samples: Vec<f32> = reader
            .samples::<f32>()
            .map(|s| s.map_err(|e| Error::AudioProcessing(e)))
            .collect::<Result<_>>()?;

        let mut params = FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 5 });

        params.set_translate(false);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let mut state = self.context.create_state()?;
        state.full(params, &samples)?;

        let num_segments = state.full_n_segments()?;
        let mut text = String::new();

        for i in 0..num_segments {
            text.push_str(&state.full_get_segment_text(i)?);
            text.push(' ');
        }

        Ok(text.trim().to_string())
    }
}
