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
        match event {
            CoreEvent::ServerCreateProgress(payload) => {
                let _ = self
                    .app_handle
                    .emit("server-create-progress", &payload);
            }
            CoreEvent::ServerStatusChanged { server_id } => {
                let _ = self
                    .app_handle
                    .emit("server-status-changed", &server_id);
            }
            CoreEvent::ImagePullProgress(payload) => {
                let _ = self.app_handle.emit("image-pull-progress", &payload);
            }
            CoreEvent::ServerLogLine { server_id, line } => {
                let _ = self
                    .app_handle
                    .emit(&format!("server-logs-{}", server_id), &line);
            }
        }
    }
}
