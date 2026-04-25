use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::error::CoreError;

#[derive(Debug, Clone, Serialize)]
pub struct ImagePullProgress {
    pub layer: Option<String>,
    pub status: String,
    pub progress: Option<String>,
}

pub async fn pull_image(
    docker: &Docker,
    image: &str,
    app_handle: &AppHandle,
) -> Result<(), CoreError> {
    let (repo, tag) = if let Some(pos) = image.rfind(':') {
        (&image[..pos], &image[pos + 1..])
    } else {
        (image, "latest")
    };

    let options = CreateImageOptions {
        from_image: repo,
        tag,
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(options), None, None);

    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                let progress = ImagePullProgress {
                    layer: info.id.clone(),
                    status: info.status.unwrap_or_default(),
                    progress: info.progress,
                };
                let _ = app_handle.emit("image-pull-progress", &progress);
            }
            Err(e) => return Err(CoreError::Docker(e)),
        }
    }

    Ok(())
}
