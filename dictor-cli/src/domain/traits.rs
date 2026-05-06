// domain/traits.rs — domain traits (interfaces)
//
// A contract: "something that can transcribe audio".
// Domain defines WHAT is needed, Infrastructure decides HOW.

use crate::domain::models::{TranscriptionResult, HealthStatus, HistoryRecord, ExportFormat};
use thiserror::Error;

/// STT operation errors.
///
/// `thiserror::Error` automatically implements Display and Error traits.
#[derive(Error, Debug)]
pub enum SttError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Server returned error: {0}")]
    ServerError(u16),

    #[error("Response parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Export error: {0}")]
    ExportError(String),
}

/// Contract for STT client.
///
/// `#[cfg_attr(test, mockall::automock)]` generates MockSttClient only when compiling tests.
#[cfg_attr(test, mockall::automock)]
pub trait SttClient: Send + Sync {
    /// Transcribes an audio file at the given path.
    fn transcribe(&self, audio_path: &str, preprompt: &str) -> Result<TranscriptionResult, SttError>;
}

/// Contract for STT server health check.
#[cfg_attr(test, mockall::automock)]
pub trait HealthCheck: Send + Sync {
    /// Checks server availability and readiness.
    fn check_health(&self) -> Result<HealthStatus, SttError>;
}

/// Contract for transcription cache.
#[cfg_attr(test, mockall::automock)]
pub trait Cache: Send + Sync {
    /// Get result by key (file hash).
    fn get(&self, key: &str) -> Option<TranscriptionResult>;

    /// Save result to cache.
    fn set(&self, key: &str, result: &TranscriptionResult) -> Result<(), SttError>;
}

/// Contract for audio file validation.
#[cfg_attr(test, mockall::automock)]
pub trait AudioValidator: Send + Sync {
    /// Verifies that the file is suitable for sending to the STT server.
    fn validate(&self, audio_path: &str) -> Result<(), SttError>;
}

/// Contract for transcription history storage.
#[cfg_attr(test, mockall::automock)]
pub trait HistoryStorage: Send + Sync {
    /// Saves a new record to history.
    fn save(&self, record: &HistoryRecord) -> Result<(), SttError>;

    /// Returns all saved records (newest first).
    fn get_all(&self) -> Result<Vec<HistoryRecord>, SttError>;

    /// Deletes a record by its database id.
    fn delete(&self, id: i64) -> Result<(), SttError>;
}

/// Contract for transcription exporters.
#[cfg_attr(test, mockall::automock)]
pub trait Exporter: Send + Sync {
    /// Converts transcription result to formatted string.
    fn export(&self, result: &TranscriptionResult, format: ExportFormat) -> Result<String, SttError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Тесты SttError
    // ============================================================

    #[test]
    fn test_error_file_not_found_message() {
        // Проверяем, что ошибка содержит путь к файлу
        let err = SttError::FileNotFound("audio.wav".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("audio.wav"));
    }

    #[test]
    fn test_error_server_error_code() {
        let err = SttError::ServerError(500);
        let msg = format!("{}", err);
        assert!(msg.contains("500"));
    }

    #[test]
    fn test_error_network_error_message() {
        let err = SttError::NetworkError("connection refused".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("connection refused"));
    }

    #[test]
    fn test_error_parse_error_message() {
        let err = SttError::ParseError("invalid json".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("invalid json"));
    }

    #[test]
    fn test_error_validation_error_message() {
        let err = SttError::ValidationError("File too large".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("File too large"));
    }

    #[test]
    fn test_error_database_error_message() {
        let err = SttError::DatabaseError("database is locked".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("database is locked"));
    }

    #[test]
    fn test_error_export_error_message() {
        let err = SttError::ExportError("format error".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("format error"));
    }

    // ============================================================
    // Тест SttClient trait через ручной mock
    // ============================================================

    #[test]
    fn test_stt_client_trait_with_manual_mock() {
        // Manual mock — struct implementing the trait.
        struct ManualMockClient;

        impl SttClient for ManualMockClient {
            fn transcribe(&self, _audio_path: &str, _preprompt: &str) -> Result<TranscriptionResult, SttError> {
                Ok(TranscriptionResult {
                    text: "test text".to_string(),
                    segments: vec![],
                    language: "ru".to_string(),
                    processing_time: 2.5,
                })
            }
        }

        let client = ManualMockClient;
        let result = client.transcribe("test.wav", "").unwrap();

        assert_eq!(result.text, "test text");
        assert_eq!(result.language, "ru");
        assert_eq!(result.processing_time, 2.5);
    }

    #[test]
    fn test_stt_client_trait_returns_error() {
        // Mock, который возвращает ошибку — проверяем обработку ошибок
        struct FailingMockClient;

        impl SttClient for FailingMockClient {
            fn transcribe(&self, audio_path: &str, _preprompt: &str) -> Result<TranscriptionResult, SttError> {
                Err(SttError::FileNotFound(audio_path.to_string()))
            }
        }

        let client = FailingMockClient;
        let result = client.transcribe("missing.wav", "");

        // .is_err() — check for error
        assert!(result.is_err());

        // Match on concrete error type
        match result.unwrap_err() {
            SttError::FileNotFound(path) => assert_eq!(path, "missing.wav"),
            other => panic!("Expected FileNotFound, got: {:?}", other),
        }
    }

    // ============================================================
    // Тест HealthCheck trait через ручной mock
    // ============================================================

    #[test]
    fn test_health_check_trait_with_manual_mock() {
        struct MockHealthCheck;

        impl HealthCheck for MockHealthCheck {
            fn check_health(&self) -> Result<HealthStatus, SttError> {
                Ok(HealthStatus {
                    status: "ok".to_string(),
                    model_loaded: true,
                })
            }
        }

        let checker = MockHealthCheck;
        let result = checker.check_health().unwrap();

        assert!(result.is_ready());
    }

    // ============================================================
    // Тест Cache trait через ручной mock
    // ============================================================

    #[test]
    fn test_cache_trait_with_manual_mock() {
        struct MockCache;

        impl Cache for MockCache {
            fn get(&self, key: &str) -> Option<TranscriptionResult> {
                if key == "hit" {
                    Some(TranscriptionResult {
                        text: "cached".to_string(),
                        segments: vec![],
                        language: "ru".to_string(),
                        processing_time: 0.1,
                    })
                } else {
                    None
                }
            }

            fn set(&self, _key: &str, _result: &TranscriptionResult) -> Result<(), SttError> {
                Ok(())
            }
        }

        let cache = MockCache;
        assert!(cache.get("miss").is_none());
        assert_eq!(cache.get("hit").unwrap().text, "cached");
    }
}
