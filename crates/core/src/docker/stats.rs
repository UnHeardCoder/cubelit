use bollard::container::StatsOptions;
use bollard::Docker;
use futures_util::StreamExt;
use serde::Serialize;

use crate::error::CoreResult;

#[derive(Debug, Clone, Serialize)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_usage_mb: f64,
    pub memory_limit_mb: f64,
}

pub async fn get_container_stats(
    docker: &Docker,
    container_id: &str,
) -> CoreResult<ContainerStats> {
    let options = StatsOptions {
        stream: false,
        one_shot: true,
    };

    let mut stream = docker.stats(container_id, Some(options));

    if let Some(Ok(stats)) = stream.next().await {
        let cpu_delta = stats
            .cpu_stats
            .cpu_usage
            .total_usage
            .saturating_sub(stats.precpu_stats.cpu_usage.total_usage)
            as f64;

        let system_delta = stats
            .cpu_stats
            .system_cpu_usage
            .unwrap_or(0)
            .saturating_sub(stats.precpu_stats.system_cpu_usage.unwrap_or(0))
            as f64;

        let num_cpus = stats
            .cpu_stats
            .online_cpus
            .unwrap_or(1) as f64;

        let cpu_percent = if system_delta > 0.0 {
            (cpu_delta / system_delta) * num_cpus * 100.0
        } else {
            0.0
        };

        let memory_usage = stats.memory_stats.usage.unwrap_or(0) as f64;
        let memory_limit = stats.memory_stats.limit.unwrap_or(0) as f64;

        Ok(ContainerStats {
            cpu_percent,
            memory_usage_mb: memory_usage / 1_048_576.0,
            memory_limit_mb: memory_limit / 1_048_576.0,
        })
    } else {
        Ok(ContainerStats {
            cpu_percent: 0.0,
            memory_usage_mb: 0.0,
            memory_limit_mb: 0.0,
        })
    }
}
