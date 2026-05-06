// application/validate.rs — Decorator for validation before sending
//
// Validates file (size, extension) first, then delegates to base client.

use crate::domain::models::TranscriptionResult;
use crate::domain::traits::{AudioValidator, SttClient, SttError};

/// STT client that validates the file (size, extension) first,
/// then passes the task down the chain.
pub struct ValidatedSttClient {
    validator: Box<dyn AudioValidator>,
    base: Box<dyn SttClient>,
}

impl ValidatedSttClient {
    /// Creates a transparent wrapper with a validator.
    pub fn new(validator: Box<dyn AudioValidator>, base: Box<dyn SttClient>) -> Self {
        Self { validator, base }
    }
}

impl SttClient for ValidatedSttClient {
    fn transcribe(&self, audio_path: &str, preprompt: &str) -> Result<TranscriptionResult, SttError> {
        self.validator.validate(audio_path)?;
        self.base.transcribe(audio_path, preprompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::traits::{MockAudioValidator, MockSttClient};

    #[test]
    fn test_transcribe_passes_when_validator_succeeds() {
        let mut mock_validator = MockAudioValidator::new();
        mock_validator.expect_validate()
            .with(mockall::predicate::eq("good.wav"))
            .times(1)
            .returning(|_| Ok(()));

        let mut mock_base = MockSttClient::new();
        mock_base.expect_transcribe()
            .times(1)
            .returning(|_, _| {
                Ok(TranscriptionResult {
                    text: "Success".to_string(),
                    segments: vec![],
                    language: "ru".to_string(),
                    processing_time: 1.0,
                })
            });

        let client = ValidatedSttClient::new(Box::new(mock_validator), Box::new(mock_base));
        let result = client.transcribe("good.wav", "");

        assert!(result.is_ok());
        assert_eq!(result.unwrap().text, "Success");
    }

    #[test]
    fn test_transcribe_fails_fast_when_validator_fails() {
        let mut mock_validator = MockAudioValidator::new();
        mock_validator.expect_validate()
            .with(mockall::predicate::eq("bad.txt"))
            .times(1)
            .returning(|_| Err(SttError::ValidationError("Wrong format".to_string())));

        let mut mock_base = MockSttClient::new();
        // IMPORTANT: base logic must NOT be called if validation fails
        mock_base.expect_transcribe().times(0);

        let client = ValidatedSttClient::new(Box::new(mock_validator), Box::new(mock_base));
        let result = client.transcribe("bad.txt", "");

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::ValidationError(msg) => assert!(msg.contains("Wrong format")),
            _ => panic!("Expected ValidationError"),
        }
    }
}
