use bollard::container::LogsOptions;
use bollard::Docker;
use futures_util::StreamExt;

use crate::events::{CoreEvent, EventSink};

// Kept for future real-time log streaming feature (currently logs are fetched
// via get_server_logs polling from src-tauri).
#[allow(dead_code)]
pub async fn stream_container_logs(
    docker: &Docker,
    container_id: &str,
    events: &dyn EventSink,
    server_id: &str,
) {
    let options = LogsOptions::<String> {
        follow: true,
        stdout: true,
        stderr: true,
        tail: "100".to_string(),
        ..Default::default()
    };

    let mut stream = docker.logs(container_id, Some(options));

    while let Some(result) = stream.next().await {
        match result {
            Ok(output) => {
                events.emit(CoreEvent::ServerLogLine {
                    server_id: server_id.to_string(),
                    line: output.to_string(),
                });
            }
            Err(_) => break,
        }
    }
}
