use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures_util::StreamExt;

use crate::error::CoreError;
use crate::events::{CoreEvent, EventSink, ImagePullProgress};

pub async fn pull_image(
    docker: &Docker,
    image: &str,
    events: &dyn EventSink,
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
                events.emit(CoreEvent::ImagePullProgress(ImagePullProgress {
                    layer: info.id.clone(),
                    status: info.status.unwrap_or_default(),
                    progress: info.progress,
                }));
            }
            Err(e) => return Err(CoreError::Docker(e)),
        }
    }

    Ok(())
}
