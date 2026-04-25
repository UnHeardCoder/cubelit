use serde::Serialize;

/// The single error type returned by every operation in `cubelit-core`.
/// Implements `Serialize` as a plain string so the existing IPC contract
/// (frontend renders errors as strings) is preserved byte-for-byte.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl Serialize for CoreError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Convenience alias used throughout core.
pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_serializes_to_string() {
        let err = CoreError::NotFound("server 42".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Not found: server 42""#);
    }

    #[test]
    fn validation_serializes_to_string() {
        let err = CoreError::Validation("port out of range".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Validation error: port out of range""#);
    }

    #[test]
    fn io_error_serializes_to_string() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = CoreError::Io(io_err);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("IO error"));
        assert!(json.contains("file missing"));
    }

    /// Lock the byte-identical Serialize output relied on by the desktop
    /// frontend — these must match the v0.1.7 strings exactly.
    #[test]
    fn serialize_format_is_byte_identical_to_v0_1_7() {
        let cases: Vec<(CoreError, &str)> = vec![
            (
                CoreError::NotFound("server-x".into()),
                "\"Not found: server-x\"",
            ),
            (
                CoreError::Validation("bad port".into()),
                "\"Validation error: bad port\"",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(
                serde_json::to_string(&err).unwrap(),
                expected,
                "v0.1.8 serialization must match v0.1.7 byte-for-byte"
            );
        }
    }
}
