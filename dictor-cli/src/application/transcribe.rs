// application/transcribe.rs — Transcribe Use Case
//
// Orchestrates transcription: takes SttClient (trait), calls transcribe().
// Doesn't know HOW transcription happens — via HTTP, FFI, or mock.

use crate::domain::models::TranscriptionResult;
use crate::domain::traits::{SttClient, SttError};

/// Use case for transcribing an audio file.
/// Uses `Box<dyn SttClient>` for heap allocation and dynamic dispatch.
pub struct TranscribeUseCase {
    client: Box<dyn SttClient>,
}

impl TranscribeUseCase {
    /// Creates use case with the given STT client.
    pub fn new(client: Box<dyn SttClient>) -> Self {
        Self { client }
    }

    /// Executes transcription of an audio file.
    ///
    /// For now — simply delegates to client.
    /// Future: validation, logging, etc.
    pub fn execute(&self, audio_path: &str, preprompt: &str) -> Result<TranscriptionResult, SttError> {
        self.client.transcribe(audio_path, preprompt)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::TranscriptionResult;
    use crate::domain::traits::{MockSttClient, SttError};

    // ============================================================
    // Test: successful transcription
    // ============================================================

    #[test]
    fn test_transcribe_success() {
        // Arrange: create mock SttClient via mockall
        // Arrange: create mock SttClient via mockall
        let mut mock = MockSttClient::new();

        // expect_transcribe() — set up call expectation
        mock.expect_transcribe()
            .withf(|path, _preprompt| path == "audio.wav")
            .times(1)
            .returning(|_, _| {
                Ok(TranscriptionResult {
                    text: "Hello world".to_string(),
                    segments: vec![],
                    language: "ru".to_string(),
                    processing_time: 3.5,
                })
            });

        let use_case = super::TranscribeUseCase::new(Box::new(mock));
        let result = use_case.execute("audio.wav", "");

        // Assert
        assert!(result.is_ok());
        let transcription = result.unwrap();
        assert_eq!(transcription.text, "Hello world");
        assert_eq!(transcription.language, "ru");
        assert_eq!(transcription.processing_time, 3.5);
    }

    // ============================================================
    // Test: file not found
    // ============================================================

    #[test]
    fn test_transcribe_file_not_found() {
        let mut mock = MockSttClient::new();

        mock.expect_transcribe()
            .withf(|path, _| path == "missing.wav")
            .times(1)
            .returning(|path, _| Err(SttError::FileNotFound(path.to_string())));

        let use_case = super::TranscribeUseCase::new(Box::new(mock));
        let result = use_case.execute("missing.wav", "");

        // Verify use case propagates error upward
        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::FileNotFound(path) => assert_eq!(path, "missing.wav"),
            other => panic!("Expected FileNotFound, got: {:?}", other),
        }
    }

    // ============================================================
    // Test: server error
    // ============================================================

    #[test]
    fn test_transcribe_server_error() {
        let mut mock = MockSttClient::new();

        mock.expect_transcribe()
            .times(1)
            .returning(|_, _| Err(SttError::ServerError(500)));

        let use_case = super::TranscribeUseCase::new(Box::new(mock));
        let result = use_case.execute("audio.wav", "");

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::ServerError(code) => assert_eq!(code, 500),
            other => panic!("Expected ServerError, got: {:?}", other),
        }
    }

    // ============================================================
    // Test: network error
    // ============================================================

    #[test]
    fn test_transcribe_network_error() {
        let mut mock = MockSttClient::new();

        mock.expect_transcribe()
            .times(1)
            .returning(|_, _| Err(SttError::NetworkError("connection refused".to_string())));

        let use_case = super::TranscribeUseCase::new(Box::new(mock));
        let result = use_case.execute("audio.wav", "");

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::NetworkError(msg) => assert!(msg.contains("connection refused")),
            other => panic!("Expected NetworkError, got: {:?}", other),
        }
    }
}
