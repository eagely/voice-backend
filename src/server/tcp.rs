use crate::error::Result;
use crate::model::command::Command;
use crate::service::runtime::runtime_service::RuntimeService;
use crate::service::{
    parsing::ParsingService, recording::RecordingService, transcription::TranscriptionService,
};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio_stream::StreamExt;

static STRING_CAP: usize = 2048;

macro_rules! arc {
    ($item: expr) => {
        Arc::new($item)
    };
}

// Inner part of the `TcpServer`
// separated to not over `Arc` ourselves.
pub struct Inner<R, T, P, RT>
where
    R: RecordingService,
    T: TranscriptionService,
    P: ParsingService,
    RT: RuntimeService,
{
    recorder: R,
    transcriber: T,
    parser: P,
    runtime: RT,
}

impl<R, T, P, RT> Inner<R, T, P, RT>
where
    R: RecordingService,
    T: TranscriptionService,
    P: ParsingService,
    RT: RuntimeService,
{
    fn arc_new(r: R, t: T, p: P, rt: RT) -> Arc<Self> {
        let me = Inner {
            recorder: r,
            transcriber: t,
            parser: p,
            runtime: rt,
        };

        arc!(me)
    }
}

pub struct TcpServer<R, T, P, RT>
where
    R: RecordingService,
    T: TranscriptionService,
    P: ParsingService,
    RT: RuntimeService,
{
    listener: TcpListener,
    inner: Arc<Inner<R, T, P, RT>>,
}

impl<R, T, P, RT> TcpServer<R, T, P, RT>
where
    R: RecordingService,
    T: TranscriptionService,
    P: ParsingService,
    RT: RuntimeService,
{
    pub fn new(addr: &str, r: R, t: T, p: P, rt: RT) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;

        Ok(Self {
            listener,
            inner: Inner::arc_new(r, t, p, rt),
        })
    }

    pub async fn listen(&self) -> Result<()> {
        let (stream, _addr) = self.listener.accept()?;
        let clone = self.inner.clone();

        // we just drop tokio's `JoinHandle`
        // but you could collect them into something
        // to await later.
        let _ = tokio::spawn(self.handle_client(stream, clone));

        Ok(())
    }

    async fn handle_client(&self, stream: TcpStream, service: Arc<Inner>) -> Result<()> {
        let mut reader = BufReader::new(&stream);
        let mut writer = &stream;

        // slightly more efficient.
        // it would only panic in case of the allocator failing.
        // and well if the allocator fails, most stuff would fail.
        // including a regular `String::new` function.
        let mut line = String::try_with_capacity(STRING_CAP).expect("memory allocation failure");
        let mut recording_active = false;

        while reader.read_line(&mut line)? > 0 {
            match line.as_str().into() {
                Command::StartRecording => {
                    service.recorder.start()?;
                    recording_active = true;
                    writeln!(writer, "Recording started.")?;
                }

                Command::StopRecording => {
                    if recording_active {
                        let audio = service.recorder.stop()?;
                        recording_active = false;

                        let transcription = service.transcriber.transcribe(&audio).await?;
                        let action = service.parser.parse(&transcription).await?;

                        let mut output_stream = service.runtime.run(action).await?;

                        // i think this could be made into a crunchy closure.
                        while let Some(output) = output_stream.next().await {
                            match output {
                                Ok(text) => writeln!(writer, "{}", text)?,
                                Err(e) => writeln!(writer, "Error: {}", e)?,
                            }
                        }

                        // since we were recording, let's just return now.
                        return Ok(());
                    }

                    writeln!(writer, "No recording in progress.")?;
                }

                Command::Unknown(command) => writeln!(writer, "Unknown command: {}", command)?,
            }

            line.clear();
        }

        Ok(())
    }
}
