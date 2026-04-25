//! Transport-agnostic progress / status events for long-running core operations.
//!
//! Core never depends on Tauri (or any other transport). Instead, every
//! function that wants to emit progress takes an `&dyn EventSink`. The
//! desktop app provides a `TauriEventSink` that maps `CoreEvent` variants to
//! Tauri event names — preserving the v0.1.7 wire format byte-for-byte. The
//! upcoming CLI uses `NoopSink` (or a stdout impl), and the HTTP agent will
//! provide a WebSocket sink.
//!
//! The payload structs (`ServerCreateProgress`, `ImagePullProgress`) live
//! here so consumers can serialize them directly when they want the legacy
//! Tauri wire format.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ServerCreateProgress {
    pub step: String,
    pub progress: Option<f32>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImagePullProgress {
    pub layer: Option<String>,
    pub status: String,
    pub progress: Option<String>,
}

/// All progress / status events core can emit. Each variant maps 1:1 to a
/// Tauri event name in `TauriEventSink`. Adding a new variant is the only
/// way to introduce a new wire-level event — the trait stays narrow.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    /// Server creation pipeline progress. Tauri event:
    /// `"server-create-progress"`.
    ServerCreateProgress(ServerCreateProgress),
    /// A server's running/stopped status changed. Tauri event:
    /// `"server-status-changed"` with the server id as payload.
    ServerStatusChanged { server_id: String },
    /// Per-layer Docker image pull progress. Tauri event:
    /// `"image-pull-progress"`.
    ImagePullProgress(ImagePullProgress),
    /// One line of container log output. Tauri event:
    /// `"server-logs-{server_id}"` with the line as payload. (Currently
    /// only used by the dead-code `stream_container_logs` helper kept for
    /// the upcoming real-time log feature.)
    ServerLogLine { server_id: String, line: String },
}

/// Sink for [`CoreEvent`]s. Implementors decide how to deliver each event
/// (Tauri IPC, WebSocket frame, stdout, drop on the floor, etc.). Impls
/// must be `Send + Sync` so background tasks can fan out to them.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: CoreEvent);
}

/// Drops every event on the floor. Useful for tests, the CLI's quiet mode,
/// or any caller that doesn't care about progress.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSink;

impl EventSink for NoopSink {
    fn emit(&self, _event: CoreEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_create_progress_wire_format_is_byte_identical_to_v0_1_7() {
        // The Tauri event payload was a flat object before workspace split.
        // After the split the same struct serializes to the exact same JSON.
        let p = ServerCreateProgress {
            step: "pulling".into(),
            progress: Some(0.2),
            message: "Pulling image foo:bar...".into(),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(
            json,
            r#"{"step":"pulling","progress":0.2,"message":"Pulling image foo:bar..."}"#
        );
    }

    #[test]
    fn image_pull_progress_wire_format_is_byte_identical_to_v0_1_7() {
        let p = ImagePullProgress {
            layer: Some("abc123".into()),
            status: "Downloading".into(),
            progress: Some("[==>  ]".into()),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(
            json,
            r#"{"layer":"abc123","status":"Downloading","progress":"[==>  ]"}"#
        );
    }

    #[test]
    fn noop_sink_swallows_events() {
        let sink = NoopSink;
        sink.emit(CoreEvent::ServerStatusChanged {
            server_id: "abc".into(),
        });
        // Nothing to assert — the test passes if the call compiles and
        // doesn't panic, proving NoopSink can be used as a default impl.
    }
}
