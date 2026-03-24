use bollard::container::LogsOptions;
use bollard::Docker;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};

#[allow(dead_code)]
pub async fn stream_container_logs(
    docker: &Docker,
    container_id: &str,
    app_handle: &AppHandle,
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
    let event_name = format!("server-logs-{}", server_id);

    while let Some(result) = stream.next().await {
        match result {
            Ok(output) => {
                let _ = app_handle.emit(&event_name, output.to_string());
            }
            Err(_) => break,
        }
    }
}
