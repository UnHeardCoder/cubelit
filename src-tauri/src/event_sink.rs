//! Tauri-backed implementation of `cubelit_core::events::EventSink`.
//!
//! Maps each `CoreEvent` variant to the exact Tauri event name + payload
//! shape that the frontend has been subscribed to since v0.1.7. The wire
//! format is preserved byte-for-byte: payload structs are re-exported
//! from core and serialized directly via `AppHandle::emit`.
use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use cubelit_core::events::{CoreEvent, EventSink};

pub struct TauriEventSink {
    app_handle: AppHandle,
}

impl TauriEventSink {
    /// Plain constructor.
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Convenience: wrap a Tauri `AppHandle` in `Arc<dyn EventSink>` so
    /// background tasks (crash watcher, readiness watcher, etc.) can hold
    /// a sink without knowing about the concrete Tauri impl.
    pub fn shared(app_handle: AppHandle) -> Arc<dyn EventSink> {
        Arc::new(Self::new(app_handle))
    }
}

impl EventSink for TauriEventSink {
    fn emit(&self, event: CoreEvent) {
        // Tauri's `Emitter::emit` returns `Result<()>`; failure means
        // serialization or the IPC transport broke. Failed emits used to
        // be discarded with `let _ = ...` — now we log at warn so the
        // failure shows up in `cubelit.log`. Events are still best-effort
        // (we don't propagate the error back into core).
        let (event_name, result) = match &event {
            CoreEvent::ServerCreateProgress(payload) => (
                "server-create-progress",
                self.app_handle.emit("server-create-progress", payload),
            ),
            CoreEvent::ServerStatusChanged { server_id } => (
                "server-status-changed",
                self.app_handle.emit("server-status-changed", server_id),
            ),
            CoreEvent::ImagePullProgress(payload) => (
                "image-pull-progress",
                self.app_handle.emit("image-pull-progress", payload),
            ),
            CoreEvent::ServerLogLine { server_id, line } => {
                let topic = format!("server-logs-{}", server_id);
                ("server-logs-{id}", self.app_handle.emit(&topic, line))
            }
        };

        if let Err(e) = result {
            tracing::warn!(
                event = event_name,
                error = %e,
                "TauriEventSink::emit failed — frontend will not see this event"
            );
        }
    }
}
