// domain/models.rs — domain entities
//
use serde::{Deserialize, Serialize};
/// Transcription segment — text fragment with timestamps.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct Segment {
    /// Segment start time (seconds)
    pub start: f64,
    /// Segment end time (seconds)
    pub end: f64,
    /// Segment text
    pub text: String,
}

/// Audio transcription result.
///
/// Matches STT server JSON response:
/// ```json
/// {
///   "text": "...",
///   "segments": [{"start": 0.0, "end": 6.0, "text": "..."}],
///   "language": "ru",
///   "processing_time": 2.195
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct TranscriptionResult {
    /// Full recognized text
    pub text: String,
    /// Segments with timestamps
    pub segments: Vec<Segment>,
    /// Audio language (e.g. "ru", "en")
    pub language: String,
    /// Processing time in seconds
    pub processing_time: f64,
}

/// Task status in transcription queue.
#[derive(Debug, Clone)]
pub enum TaskStatus {
    /// Pending processing
    Pending,
    /// Processing
    Processing,
    /// Completed successfully
    Completed(TranscriptionResult),
    /// Failed with error
    Failed(String),
}

/// Transcription task in queue.
#[derive(Debug, Clone)]
pub struct TranscriptionTask {
    /// Unique task ID (sequential number)
    pub id: usize,
    /// Path to audio file
    pub audio_path: String,
    /// Current task status
    pub status: TaskStatus,
}

impl TranscriptionTask {
    /// Creates a new task in Pending status.
    pub fn new(id: usize, audio_path: &str) -> Self {
        Self {
            id,
            audio_path: audio_path.to_string(),
            status: TaskStatus::Pending,
        }
    }

    /// Returns whether the task is finished (success or error).
    pub fn is_finished(&self) -> bool {
        matches!(self.status, TaskStatus::Completed(_) | TaskStatus::Failed(_))
    }
}

/// STT server health status.
///
/// Matches JSON response from `/health`:
/// ```json
/// {
///   "status": "ok",
///   "model_loaded": true
/// }
/// ```
#[derive(Debug, Clone, Deserialize, PartialEq, uniffi::Record)]
pub struct HealthStatus {
    pub status: String,
    pub model_loaded: bool,
}

impl HealthStatus {
    /// Returns whether the server is fully ready.
    pub fn is_ready(&self) -> bool {
        self.status == "ok" && self.model_loaded
    }
}

/// Transcription history record for SQLite storage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, uniffi::Record)]
pub struct HistoryRecord {
    /// Database primary key
    pub id: Option<i64>, 
    /// Path to the audio file
    pub audio_path: String,
    /// Recognized text
    pub text: String,
    /// Language
    pub language: String,
    /// Processing duration (seconds)
    pub processing_time: f64,
    /// Creation time (Unix timestamp)
    pub created_at: i64,
}

/// Application config passed at initialization.
#[derive(Debug, Clone, uniffi::Record)]
pub struct AppConfig {
    pub server_address: String,
    pub preprompt: String,
}

/// Transcription export format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    /// Plain text
    Txt,
    /// SRT subtitles
    Srt,
    /// Raw JSON (TranscriptionResult structure)
    Json,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_transcription_result_creation() {
        let result = super::TranscriptionResult {
            text: "Hello world".to_string(),
            segments: vec![
                super::Segment {
                    start: 0.0,
                    end: 3.5,
                    text: "Hello world".to_string(),
                },
            ],
            language: "ru".to_string(),
            processing_time: 2.1,
        };

        assert_eq!(result.text, "Hello world");
        assert_eq!(result.language, "ru");
        assert_eq!(result.processing_time, 2.1);
        assert_eq!(result.segments.len(), 1);
        assert_eq!(result.segments[0].start, 0.0);
        assert_eq!(result.segments[0].end, 3.5);
    }

    #[test]
    fn test_transcription_result_debug_display() {
        let result = super::TranscriptionResult {
            text: "Hello".to_string(),
            segments: vec![],
            language: "en".to_string(),
            processing_time: 1.0,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("Hello"));
    }

    #[test]
    fn test_deserialize_from_json() {
        // Verify parsing of real server JSON
        let json = r#"{
            "text": "Hello",
            "segments": [{"start": 0.0, "end": 2.0, "text": "Hello"}],
            "language": "ru",
            "processing_time": 1.5
        }"#;

        let result: super::TranscriptionResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.text, "Hello");
        assert_eq!(result.language, "ru");
        assert_eq!(result.processing_time, 1.5);
        assert_eq!(result.segments.len(), 1);
    }

    // ============================================================
    // Тесты TaskStatus и TranscriptionTask (очередь)
    // ============================================================

    #[test]
    fn test_task_creation_has_pending_status() {
        // Новая задача всегда создаётся в статусе Pending
        let task = super::TranscriptionTask::new(0, "audio.wav");
        assert_eq!(task.id, 0);
        assert_eq!(task.audio_path, "audio.wav");
        assert!(matches!(task.status, super::TaskStatus::Pending));
    }

    #[test]
    fn test_task_is_not_finished_when_pending() {
        let task = super::TranscriptionTask::new(0, "audio.wav");
        assert!(!task.is_finished());
    }

    #[test]
    fn test_task_is_not_finished_when_processing() {
        let mut task = super::TranscriptionTask::new(0, "audio.wav");
        task.status = super::TaskStatus::Processing;
        assert!(!task.is_finished());
    }

    #[test]
    fn test_task_is_finished_when_completed() {
        let mut task = super::TranscriptionTask::new(0, "audio.wav");
        task.status = super::TaskStatus::Completed(super::TranscriptionResult {
            text: "test".to_string(),
            segments: vec![],
            language: "en".to_string(),
            processing_time: 1.0,
        });
        assert!(task.is_finished());
    }

    #[test]
    fn test_task_is_finished_when_failed() {
        let mut task = super::TranscriptionTask::new(0, "audio.wav");
        task.status = super::TaskStatus::Failed("error".to_string());
        assert!(task.is_finished());
    }

    // ============================================================
    // Тесты HealthStatus
    // ============================================================

    #[test]
    fn test_health_status_is_ready_when_ok_and_loaded() {
        let status = super::HealthStatus {
            status: "ok".to_string(),
            model_loaded: true,
        };
        assert!(status.is_ready());
    }

    #[test]
    fn test_health_status_is_not_ready_when_not_ok() {
        let status = super::HealthStatus {
            status: "error".to_string(),
            model_loaded: true,
        };
        assert!(!status.is_ready());
    }

    #[test]
    fn test_health_status_is_not_ready_when_model_unloaded() {
        let status = super::HealthStatus {
            status: "ok".to_string(),
            model_loaded: false,
        };
        assert!(!status.is_ready());
    }

    #[test]
    fn test_health_status_deserialize() {
        let json = r#"{"status": "ok", "model_loaded": true}"#;
        let status: super::HealthStatus = serde_json::from_str(json).unwrap();
        
        assert_eq!(status.status, "ok");
        assert!(status.model_loaded);
        assert!(status.is_ready());
    }

    // ============================================================
    // Тесты HistoryRecord
    // ============================================================

    #[test]
    fn test_history_record_creation() {
        let record = super::HistoryRecord {
            id: None,
            audio_path: "/test/audio.wav".to_string(),
            text: "History".to_string(),
            language: "ru".to_string(),
            processing_time: 2.0,
            created_at: 1700000000,
        };

        assert_eq!(record.id, None);
        assert_eq!(record.audio_path, "/test/audio.wav");
        assert_eq!(record.text, "History");
    }
}
