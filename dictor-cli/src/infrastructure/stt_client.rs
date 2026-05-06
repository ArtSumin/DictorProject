// infrastructure/stt_client.rs — HTTP client for the STT server
//
// Implements SttClient trait from domain — dependency inversion.

use std::time::Duration;
use reqwest::blocking::Client;
use crate::domain::models::{TranscriptionResult, HealthStatus};
use crate::domain::traits::{SttClient, SttError, HealthCheck};

/// HTTP client for the Python STT server.
///
/// - `Client` — reuses TCP connections
/// - `base_url` — server URL, e.g. "http://localhost:8000"
pub struct HttpSttClient {
    /// HTTP client
    client: Client,
    /// Base URL of the STT server
    pub base_url: String,
}

impl HttpSttClient {
    /// Creates a client with the given server URL.
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client,
            base_url: base_url.to_string(),
        }
    }
}

impl SttClient for HttpSttClient {
    /// Sends an audio file to the STT server and returns the result.
    fn transcribe(&self, audio_path: &str, preprompt: &str) -> Result<TranscriptionResult, SttError> {
        let file_bytes = std::fs::read(audio_path)
            .map_err(|e| SttError::FileNotFound(format!("{}: {}", audio_path, e)))?;

        let file_name = std::path::Path::new(audio_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav")
            .to_string();

        let part = reqwest::blocking::multipart::Part::bytes(file_bytes)
            .file_name(file_name);
        let mut form = reqwest::blocking::multipart::Form::new()
            .part("file", part);

        if !preprompt.is_empty() {
            form = form.text("prompt", preprompt.to_string());
        }

        // 4. Send POST request
        let response = self.client
            .post(format!("{}/transcribe", self.base_url))
            .multipart(form)
            .send()
            .map_err(|e| SttError::NetworkError(e.to_string()))?;

        // 5. Check HTTP status
        if !response.status().is_success() {
            return Err(SttError::ServerError(response.status().as_u16()));
        }

        // 6. Parse JSON response
        let result: TranscriptionResult = response.json()
            .map_err(|e| SttError::ParseError(e.to_string()))?;

        Ok(result)
    }
}

impl HealthCheck for HttpSttClient {
    /// Checks server health (GET /health).
    fn check_health(&self) -> Result<HealthStatus, SttError> {
        let response = self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .map_err(|e| SttError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SttError::ServerError(response.status().as_u16()));
        }

        let status: HealthStatus = response.json()
            .map_err(|e| SttError::ParseError(e.to_string()))?;

        Ok(status)
    }
}


#[cfg(test)]
mod tests {
    use crate::domain::traits::{SttClient, SttError, HealthCheck};

    // ============================================================
    // Test: client creation
    // ============================================================

    #[test]
    fn test_client_creation() {
        // Just verify the client is created without panic
        let client = super::HttpSttClient::new("http://localhost:8000");

        // base_url is stored correctly
        assert_eq!(client.base_url, "http://localhost:8000");
    }

    // ============================================================
    // Test: file not found (no server required!)
    // ============================================================

    #[test]
    fn test_transcribe_file_not_found() {
        let client = super::HttpSttClient::new("http://localhost:8000");

        // Try to transcribe a non-existent file
        let result = client.transcribe("nonexistent_file_12345.wav", "");

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::FileNotFound(msg) => {
                assert!(msg.contains("nonexistent_file_12345.wav"));
            }
            other => panic!("Expected FileNotFound, got: {:?}", other),
        }
    }

    // ============================================================
    // Test: network error when server is down (no server required!)
    // ============================================================

    #[test]
    fn test_transcribe_network_error_when_server_down() {
        // Connect to a port where nothing is listening
        let client = super::HttpSttClient::new("http://127.0.0.1:19999");

        // Create a temp file to pass the file existence check
        let tmp_dir = std::env::temp_dir();
        let tmp_file = tmp_dir.join("dictor_test_audio.wav");
        std::fs::write(&tmp_file, b"fake audio data").unwrap();

        let result = client.transcribe(tmp_file.to_str().unwrap(), "");

        // Cleanup
        let _ = std::fs::remove_file(&tmp_file);

        // Should get a network error, not FileNotFound
        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::NetworkError(msg) => {
                assert!(!msg.is_empty(), "Error message must not be empty");
            }
            other => panic!("Expected NetworkError, got: {:?}", other),
        }
    }

    // ============================================================
    // Test: client implements SttClient trait
    // ============================================================

    #[test]
    fn test_implements_stt_client_trait() {
        // Verify HttpSttClient can be used as dyn SttClient
        // Trait object: Box<dyn SttClient>
        let client = super::HttpSttClient::new("http://localhost:8000");
        let _trait_object: Box<dyn SttClient> = Box::new(client);
        // If it compiles — trait is implemented
    }

    // ============================================================
    // Test: network error on health check
    // ============================================================

    #[test]
    fn test_check_health_network_error_when_server_down() {
        let client = super::HttpSttClient::new("http://127.0.0.1:19999");
        let result = client.check_health(); // HealthCheck trait

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::NetworkError(msg) => {
                assert!(!msg.is_empty());
            }
            other => panic!("Expected NetworkError, got: {:?}", other),
        }
    }
}
