use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures_util::StreamExt;

use crate::error::CoreError;
use crate::events::{CoreEvent, EventSink, ImagePullProgress};

/// Split a Docker image reference into `(repo, tag)`. Defaults to `"latest"`.
///
/// Naive `rfind(':')` mis-parses references whose registry component
/// contains a port (`localhost:5000/itzg/minecraft-server:java25`) — the
/// final colon would be ambiguous with the registry colon. The tag
/// separator only applies to a colon **after** the last `/` in the
/// reference (i.e. inside the path/name component, not the host).
fn split_image_ref(image: &str) -> (&str, &str) {
    // The colon that separates repo from tag must come AFTER the last `/`
    // (registry/path/name segments use `/`; the registry's own port colon
    // appears BEFORE any `/`).
    let last_slash = image.rfind('/').map(|i| i + 1).unwrap_or(0);
    if let Some(rel_pos) = image[last_slash..].rfind(':') {
        let pos = last_slash + rel_pos;
        (&image[..pos], &image[pos + 1..])
    } else {
        (image, "latest")
    }
}

#[tracing::instrument(skip(docker, events), fields(image = %image))]
pub async fn pull_image(
    docker: &Docker,
    image: &str,
    events: &dyn EventSink,
) -> Result<(), CoreError> {
    let (repo, tag) = split_image_ref(image);

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

#[cfg(test)]
mod tests {
    use super::split_image_ref;

    #[test]
    fn defaults_to_latest_when_no_tag() {
        assert_eq!(split_image_ref("itzg/minecraft-server"), ("itzg/minecraft-server", "latest"));
        assert_eq!(split_image_ref("alpine"), ("alpine", "latest"));
    }

    #[test]
    fn parses_simple_tag() {
        assert_eq!(
            split_image_ref("itzg/minecraft-server:java25"),
            ("itzg/minecraft-server", "java25")
        );
        assert_eq!(split_image_ref("alpine:3.20"), ("alpine", "3.20"));
    }

    #[test]
    fn parses_registry_with_port() {
        // The registry colon (after `localhost`) must NOT be mistaken
        // for a tag separator.
        assert_eq!(
            split_image_ref("localhost:5000/itzg/minecraft-server:java25"),
            ("localhost:5000/itzg/minecraft-server", "java25")
        );
        assert_eq!(
            split_image_ref("ghcr.io:443/foo/bar:v1"),
            ("ghcr.io:443/foo/bar", "v1")
        );
    }

    #[test]
    fn registry_with_port_no_tag_defaults_to_latest() {
        assert_eq!(
            split_image_ref("localhost:5000/itzg/minecraft-server"),
            ("localhost:5000/itzg/minecraft-server", "latest")
        );
    }
}
