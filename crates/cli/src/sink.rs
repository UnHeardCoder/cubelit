use std::io::Write;
use std::sync::{Arc, Mutex};

use cubelit_core::events::{CoreEvent, EventSink, ImagePullProgress, ServerCreateProgress};

/// Forwards core events to stdout/stderr; inject `Write` impls in tests.
pub struct CliEventSink {
    stdout: Arc<Mutex<Box<dyn Write + Send>>>,
    stderr: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl CliEventSink {
    pub fn stdio() -> Self {
        Self {
            stdout: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
            stderr: Arc::new(Mutex::new(Box::new(std::io::stderr()))),
        }
    }

    fn short_id(id: &str) -> String {
        id.chars().take(8).collect()
    }
}

impl EventSink for CliEventSink {
    fn emit(&self, event: CoreEvent) {
        match event {
            CoreEvent::ServerCreateProgress(ServerCreateProgress {
                step,
                progress,
                message,
            }) => {
                let pct = progress
                    .map(|p| format!(" ({:.0}%)", p * 100.0))
                    .unwrap_or_default();
                let line = format!("[{}] {}{}", step, message, pct);
                let mut err = self.stderr.lock().expect("stderr mutex poisoned");
                let _ = write!(err, "\r\x1b[K{}", line);
                let _ = err.flush();
                if step == "ready" {
                    let _ = writeln!(err);
                    let _ = err.flush();
                }
            }
            CoreEvent::ImagePullProgress(ImagePullProgress {
                layer,
                status,
                progress,
            }) => {
                let layer_s = layer.as_deref().unwrap_or("—");
                let prog = progress.as_deref().unwrap_or("");
                let line = format!("pulling {}: {} {}", layer_s, status, prog);
                let mut err = self.stderr.lock().expect("stderr mutex poisoned");
                let _ = write!(err, "\r\x1b[K{}", line.trim_end());
                let _ = err.flush();
            }
            CoreEvent::ServerStatusChanged { server_id } => {
                let mut err = self.stderr.lock().expect("stderr mutex poisoned");
                let _ = writeln!(
                    err,
                    "status: {} updated",
                    Self::short_id(&server_id)
                );
                let _ = err.flush();
            }
            CoreEvent::ServerLogLine { line, .. } => {
                let mut out = self.stdout.lock().expect("stdout mutex poisoned");
                let _ = writeln!(out, "{}", line.trim_end());
                let _ = out.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type CapturedStreams = (CliEventSink, Arc<Mutex<Vec<u8>>>, Arc<Mutex<Vec<u8>>>);

    struct VecWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for VecWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn sink_with_vecs() -> CapturedStreams {
        let out = Arc::new(Mutex::new(Vec::new()));
        let err = Arc::new(Mutex::new(Vec::new()));
        let sink = CliEventSink {
            stdout: Arc::new(Mutex::new(Box::new(VecWriter(Arc::clone(&out))))),
            stderr: Arc::new(Mutex::new(Box::new(VecWriter(Arc::clone(&err))))),
        };
        (sink, out, err)
    }

    #[test]
    fn create_progress_ready_emits_stderr_and_newline() {
        let (sink, _out, err) = sink_with_vecs();
        sink.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "pulling".into(),
            progress: Some(0.5),
            message: "test".into(),
        }));
        sink.emit(CoreEvent::ServerCreateProgress(ServerCreateProgress {
            step: "ready".into(),
            progress: Some(1.0),
            message: "done".into(),
        }));
        let bytes = err.lock().unwrap().clone();
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("pulling"));
        assert!(s.contains("ready"));
        assert!(s.contains('\n'));
    }

    #[test]
    fn image_pull_progress_goes_to_stderr() {
        let (sink, out, err) = sink_with_vecs();
        sink.emit(CoreEvent::ImagePullProgress(ImagePullProgress {
            layer: Some("sha:abc".into()),
            status: "Pulling".into(),
            progress: None,
        }));
        assert!(out.lock().unwrap().is_empty());
        assert!(!err.lock().unwrap().is_empty());
    }

    #[test]
    fn server_status_changed_stderr() {
        let (sink, out, err) = sink_with_vecs();
        sink.emit(CoreEvent::ServerStatusChanged {
            server_id: "abcd-efgh-ijkl".into(),
        });
        assert!(out.lock().unwrap().is_empty());
        let err_bytes = err.lock().unwrap().clone();
        let s = String::from_utf8_lossy(&err_bytes);
        assert!(s.contains("abcd-efg"));
    }

    #[test]
    fn server_log_line_stdout() {
        let (sink, out, err) = sink_with_vecs();
        sink.emit(CoreEvent::ServerLogLine {
            server_id: "x".into(),
            line: "hello log".into(),
        });
        assert!(err.lock().unwrap().is_empty());
        let out_bytes = out.lock().unwrap().clone();
        let s = String::from_utf8_lossy(&out_bytes);
        assert!(s.contains("hello log"));
    }
}
