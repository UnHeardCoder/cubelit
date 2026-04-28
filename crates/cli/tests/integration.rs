//! Docker-backed integration tests (opt-in).
//!
//! Run with Docker available:
//!   `CUBELIT_INTEGRATION_TESTS=1 cargo test -p cubelit-cli -- --ignored`
//!
//! Tests allocate real containers; clean up with `cubelit server remove --yes`.

#[test]
#[ignore]
fn install_start_stop_remove_minecraft() {
    // Scaffold for a future full flow; requires Docker + network for image pull.
}
