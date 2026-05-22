use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tabled::{Table, Tabled};

use cubelit_core::error::{CoreError, CoreResult};
use cubelit_core::smoke::{run_smoke, SmokeOptions, SmokeOutcome};

use crate::context::Context;
use crate::sink::CliEventSink;

#[derive(Tabled)]
struct Row {
    #[tabled(rename = "GAME")]
    recipe_id: String,
    #[tabled(rename = "IMAGE")]
    image: String,
    #[tabled(rename = "OUTCOME")]
    outcome: String,
    #[tabled(rename = "DURATION")]
    duration: String,
    #[tabled(rename = "LAST LOG")]
    last_log: String,
}

pub async fn run(
    ctx: &Context,
    games: Vec<String>,
    all: bool,
    keep_on_fail: bool,
    port_offset: u16,
    timeout_secs: u64,
    json_path: Option<PathBuf>,
) -> CoreResult<()> {
    if games.is_empty() && !all {
        return Err(CoreError::Validation(
            "Specify at least one --game <recipe_id> or use --all".into(),
        ));
    }

    let recipe_ids = if all { vec![] } else { games };

    let opts = SmokeOptions {
        recipe_ids,
        overall_timeout: Duration::from_secs(timeout_secs),
        keep_on_failure: keep_on_fail,
        port_offset,
        parallel: 1,
    };

    // Use CliEventSink so image-pull progress is visible on stderr.
    let events: Arc<dyn cubelit_core::events::EventSink> = Arc::new(CliEventSink::stdio());

    eprintln!("Starting smoke test — this may take a while for large images (CS2, ARK).");

    let report = run_smoke(&ctx.host, opts, events).await?;

    // Build the display table.
    let mut rows: Vec<Row> = Vec::new();
    for result in &report.results {
        let (outcome_label, duration_str, last_log) = match &result.outcome {
            SmokeOutcome::Ready { duration } => (
                "ready ✓".to_string(),
                format!("{}s", duration.as_secs()),
                String::new(),
            ),
            SmokeOutcome::Started { duration } => (
                "started (no pattern)".to_string(),
                format!("{}s", duration.as_secs()),
                String::new(),
            ),
            SmokeOutcome::ImagePullFailed(msg) => (
                "PULL FAILED".to_string(),
                "-".to_string(),
                truncate(msg, 60),
            ),
            SmokeOutcome::ContainerCrashed { last_logs } => (
                "CRASHED".to_string(),
                "-".to_string(),
                last_logs.last().map(|l| truncate(l, 60)).unwrap_or_default(),
            ),
            SmokeOutcome::Timeout { last_logs } => (
                "TIMEOUT".to_string(),
                "-".to_string(),
                last_logs.last().map(|l| truncate(l, 60)).unwrap_or_default(),
            ),
        };

        rows.push(Row {
            recipe_id: result.recipe_id.clone(),
            image: result.image.clone(),
            outcome: outcome_label,
            duration: duration_str,
            last_log,
        });
    }

    println!("\n{}", Table::new(rows));

    // Optional extra JSON output path.
    if let Some(path) = json_path {
        write_json_report(&report, &path);
    }

    // Exit non-zero if any result is failing.
    if !report.all_passing() {
        let failing: Vec<&str> = report
            .results
            .iter()
            .filter(|r| !r.outcome.is_passing())
            .map(|r| r.recipe_id.as_str())
            .collect();
        return Err(CoreError::Validation(format!(
            "Smoke test failed for: {}",
            failing.join(", ")
        )));
    }

    eprintln!("All smoke tests passed.");
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

fn write_json_report(report: &cubelit_core::smoke::SmokeReport, path: &PathBuf) {
    let rows: Vec<serde_json::Value> = report
        .results
        .iter()
        .map(|r| {
            serde_json::json!({
                "recipe_id": r.recipe_id,
                "image": r.image,
                "outcome": r.outcome.label(),
                "passing": r.outcome.is_passing(),
            })
        })
        .collect();

    let json = serde_json::json!({
        "started_at": report.started_at.to_rfc3339(),
        "finished_at": report.finished_at.to_rfc3339(),
        "results": rows,
    });

    match serde_json::to_string_pretty(&json) {
        Ok(s) => {
            if let Err(e) = std::fs::write(path, s) {
                eprintln!("warning: could not write JSON report to {}: {}", path.display(), e);
            } else {
                eprintln!("JSON report written to {}", path.display());
            }
        }
        Err(e) => eprintln!("warning: could not serialize JSON report: {}", e),
    }
}
