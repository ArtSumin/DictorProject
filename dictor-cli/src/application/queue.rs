// application/queue.rs — Use Case: transcription queue
//
// Takes a list of files, processes them sequentially,
// returns result for each file.
// Key points:
//   One error does not stop the entire queue.
//   Each task finishes as Completed or Failed.

use crate::domain::models::{TranscriptionTask, TaskStatus};
use crate::domain::traits::SttClient;

/// Use Case: processing queue of audio files.
/// Accepts SttClient trait via DI.
/// Processes files sequentially — one after another.
pub struct TranscriptionQueue {
    client: Box<dyn SttClient>,
}

impl TranscriptionQueue {
    /// Creates queue with the given STT client.
    pub fn new(client: Box<dyn SttClient>) -> Self {
        Self { client }
    }

    /// Process list of audio files sequentially.
    /// - Each file gets an ID (sequential number).
    /// - Success → Completed(result), error → Failed(message).
    /// - One error does NOT stop the queue.
    pub fn process(&self, paths: &[String], preprompt: &str) -> Vec<TranscriptionTask> {
        paths
            .iter()
            .enumerate()
            .map(|(id, path)| {
                let mut task = TranscriptionTask::new(id, path);
                task.status = TaskStatus::Processing;

                match self.client.transcribe(path, preprompt) {
                    Ok(result) => {
                        task.status = TaskStatus::Completed(result);
                    }
                    Err(err) => {
                        task.status = TaskStatus::Failed(err.to_string());
                    }
                }

                task
            })
            .collect()       // Like Array(...) — collects iterator into Vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::TranscriptionResult;
    use crate::domain::traits::{MockSttClient, SttError};

    // ============================================================
    // Test: empty queue
    // ============================================================

    #[test]
    fn test_empty_queue_returns_empty_results() {
        let mock = MockSttClient::new();
        let queue = TranscriptionQueue::new(Box::new(mock));

        let results = queue.process(&[], "");
        assert!(results.is_empty());
    }

    // ============================================================
    // Test: single file — success
    // ============================================================

    #[test]
    fn test_single_file_success() {
        let mut mock = MockSttClient::new();
        mock.expect_transcribe()
            .times(1)
            .returning(|_, _| {
                Ok(TranscriptionResult {
                    text: "Hello".to_string(),
                    segments: vec![],
                    language: "ru".to_string(),
                    processing_time: 1.0,
                })
            });

        let queue = TranscriptionQueue::new(Box::new(mock));
        let results = queue.process(&["audio.wav".to_string()], "");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 0);
        assert_eq!(results[0].audio_path, "audio.wav");
        assert!(results[0].is_finished());

        // Verify Completed contains text
        match &results[0].status {
            TaskStatus::Completed(r) => assert_eq!(r.text, "Hello"),
            other => panic!("Expected Completed, got: {:?}", other),
        }
    }

    // ============================================================
    // Test: single file — error (does not stop queue)
    // ============================================================

    #[test]
    fn test_single_file_failure() {
        let mut mock = MockSttClient::new();
        mock.expect_transcribe()
            .times(1)
            .returning(|_, _| Err(SttError::FileNotFound("missing.wav".to_string())));

        let queue = TranscriptionQueue::new(Box::new(mock));
        let results = queue.process(&["missing.wav".to_string()], "");

        assert_eq!(results.len(), 1);
        assert!(results[0].is_finished());

        match &results[0].status {
            TaskStatus::Failed(msg) => assert!(msg.contains("missing.wav")),
            other => panic!("Expected Failed, got: {:?}", other),
        }
    }

    // ============================================================
    // Test: three files — 2 successes + 1 error
    // ============================================================

    #[test]
    fn test_multiple_files_mixed_results() {
        let mut mock = MockSttClient::new();

        // Configure mock: files 1 and 3 — success, file 2 — error
        mock.expect_transcribe()
            .times(3)
            .returning(|path, _| {
                if path == "bad.wav" {
                    Err(SttError::NetworkError("timeout".to_string()))
                } else {
                    Ok(TranscriptionResult {
                        text: format!("text for {}", path),
                        segments: vec![],
                        language: "en".to_string(),
                        processing_time: 0.5,
                    })
                }
            });

        let queue = TranscriptionQueue::new(Box::new(mock));
        let results = queue.process(&[
            "a.wav".to_string(),
            "bad.wav".to_string(),
            "b.wav".to_string(),
        ], "");

        // All 3 tasks processed (queue did not stop!)
        assert_eq!(results.len(), 3);

        // File 0: success
        assert!(matches!(&results[0].status, TaskStatus::Completed(_)));
        // File 1: error
        assert!(matches!(&results[1].status, TaskStatus::Failed(_)));
        // File 2: success (queue continued after error!)
        assert!(matches!(&results[2].status, TaskStatus::Completed(_)));

        // Verify IDs
        assert_eq!(results[0].id, 0);
        assert_eq!(results[1].id, 1);
        assert_eq!(results[2].id, 2);
    }

    // ============================================================
    // Test: all files — error
    // ============================================================

    #[test]
    fn test_all_files_fail() {
        let mut mock = MockSttClient::new();
        mock.expect_transcribe()
            .times(2)
            .returning(|_, _| Err(SttError::ServerError(500)));

        let queue = TranscriptionQueue::new(Box::new(mock));
        let results = queue.process(&["a.wav".to_string(), "b.wav".to_string()], "");

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|t| matches!(&t.status, TaskStatus::Failed(_))));
    }
}
