// application/retry.rs — Use Case: Retry logic (decorator)
//
// Decorator pattern. Wrapper around the base service.
// Retries on transient failures (network, server errors).

use crate::domain::models::TranscriptionResult;
use crate::domain::traits::{SttClient, SttError};
use std::thread;
use std::time::Duration;

/// STT client with automatic retry on failures.
/// Implements the same `SttClient` trait, allowing transparent
/// implementation swapping via Dependency Injection.
pub struct RetrySttClient {
    /// Base STT client (HttpSttClient or any other)
    base: Box<dyn SttClient>,
    /// Maximum number of attempts (including the first one).
    /// Value 3 means: 1 primary + 2 retries.
    max_attempts: usize,
    /// Delay between attempts (in milliseconds)
    delay_ms: u64,
}

impl RetrySttClient {
    /// Creates a new Retry decorator.
    pub fn new(base: Box<dyn SttClient>, max_attempts: usize, delay_ms: u64) -> Self {
        Self {
            base,
            max_attempts,
            delay_ms,
        }
    }

    /// Checks whether it makes sense to retry the request for this error.
    fn should_retry(err: &SttError) -> bool {
        matches!(err, SttError::NetworkError(_) | SttError::ServerError(_))
    }
}

impl SttClient for RetrySttClient {
    fn transcribe(&self, audio_path: &str, preprompt: &str) -> Result<TranscriptionResult, SttError> {
        let mut attempt = 0;

        loop {
            attempt += 1;

            match self.base.transcribe(audio_path, preprompt) {
                Ok(result) => return Ok(result), // Success!
                Err(err) => {
                    // If not a retryable error (e.g. file not found) — fail immediately.
                    if !Self::should_retry(&err) {
                        return Err(err);
                    }

                    // If we've exhausted retry limit — return the last error.
                    if attempt >= self.max_attempts {
                        return Err(err);
                    }

                    // Wait before the next retry (only if delay > 0)
                    if self.delay_ms > 0 {
                        thread::sleep(Duration::from_millis(self.delay_ms));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::traits::MockSttClient;

    // ============================================================
    // Test 1: Success on first attempt (no retry needed)
    // ============================================================
    #[test]
    fn test_success_first_attempt() {
        let mut mock = MockSttClient::new();
        
        // Expect exactly 1 call
        mock.expect_transcribe()
            .times(1)
            .returning(|_, _| {
                Ok(TranscriptionResult {
                    text: "Test".to_string(),
                    segments: vec![],
                    language: "en".to_string(),
                    processing_time: 1.0,
                })
            });

        let client = RetrySttClient::new(Box::new(mock), 3, 0);
        let result = client.transcribe("audio.wav", "");

        assert!(result.is_ok());
        assert_eq!(result.unwrap().text, "Test");
    }

    // ============================================================
    // Test 2: Success after 1 failure (retry works)
    // ============================================================
    #[test]
    fn test_success_after_one_failure() {
        let mut mock = MockSttClient::new();
        
        // Expect exactly 2 calls
        let mut call_count = 0;
        mock.expect_transcribe()
            .times(2)
            .returning(move |_, _| {
                call_count += 1;
                if call_count == 1 {
                    Err(SttError::NetworkError("Timeout".to_string()))
                } else {
                    Ok(TranscriptionResult {
                        text: "Recovered".to_string(),
                        segments: vec![],
                        language: "en".to_string(),
                        processing_time: 1.0,
                    })
                }
            });

        let client = RetrySttClient::new(Box::new(mock), 3, 0);
        let result = client.transcribe("audio.wav", "");

        assert!(result.is_ok());
        assert_eq!(result.unwrap().text, "Recovered");
    }

    // ============================================================
    // Test 3: Exhausted attempts (fails after max_attempts)
    // ============================================================
    #[test]
    fn test_fails_after_max_attempts() {
        let mut mock = MockSttClient::new();
        
        // Max 3 attempts. Client will call the method exactly 3 times.
        mock.expect_transcribe()
            .times(3)
            .returning(|_, _| Err(SttError::ServerError(502)));

        let client = RetrySttClient::new(Box::new(mock), 3, 0);
        let result = client.transcribe("audio.wav", "");

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::ServerError(code) => assert_eq!(code, 502),
            _ => panic!("Expected ServerError"),
        }
    }

    // ============================================================
    // Test 4: No retries for fatal errors (FileNotFound, ParseError)
    // ============================================================
    #[test]
    fn test_no_retries_for_fatal_errors() {
        let mut mock = MockSttClient::new();
        
        // Despite 3 attempts in the decorator, call should be only 1,
        // since FileNotFound is not retried.
        mock.expect_transcribe()
            .times(1)
            .returning(|_, _| Err(SttError::FileNotFound("missing.wav".to_string())));

        let client = RetrySttClient::new(Box::new(mock), 3, 0);
        let result = client.transcribe("audio.wav", "");

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::FileNotFound(_) => (),
            _ => panic!("Expected FileNotFound"),
        }
    }
}
