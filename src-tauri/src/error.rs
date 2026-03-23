use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
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

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_serializes_to_string() {
        let err = AppError::NotFound("server 42".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Not found: server 42""#);
    }

    #[test]
    fn validation_serializes_to_string() {
        let err = AppError::Validation("port out of range".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Validation error: port out of range""#);
    }

    #[test]
    fn io_error_serializes_to_string() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = AppError::Io(io_err);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("IO error"));
        assert!(json.contains("file missing"));
    }
}
