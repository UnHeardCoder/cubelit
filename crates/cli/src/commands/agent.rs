use cubelit_core::error::{CoreError, CoreResult};

pub fn start_stub() -> CoreResult<()> {
    Err(CoreError::Validation(
        "Agent not yet implemented. Coming in v0.3.0.".into(),
    ))
}
