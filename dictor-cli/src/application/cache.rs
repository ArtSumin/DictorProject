// application/cache.rs — Use Case: Caching results
//
// Decorator pattern: computes SHA-256 file hash.
// If hash is in cache — returns result (no network).
// If not — calls base SttClient, saves to cache, returns result.

use crate::domain::models::TranscriptionResult;
use crate::domain::traits::{Cache, SttClient, SttError};
use sha2::{Digest, Sha256};

/// STT client that caches results.
/// Saves time and resources on repeated requests
/// for the same file.
pub struct CachedSttClient {
    base: Box<dyn SttClient>,
    cache: Box<dyn Cache>,
}

impl CachedSttClient {
    /// Creates a decorator with base STT client and cache.
    pub fn new(base: Box<dyn SttClient>, cache: Box<dyn Cache>) -> Self {
        Self { base, cache }
    }

    /// Computes SHA-256 hash of the file.
    fn compute_file_hash(path: &str) -> Result<String, SttError> {
        let bytes = std::fs::read(path)
            .map_err(|e| SttError::FileNotFound(format!("For hashing: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

impl SttClient for CachedSttClient {
    fn transcribe(&self, audio_path: &str, preprompt: &str) -> Result<TranscriptionResult, SttError> {
        let file_hash = Self::compute_file_hash(audio_path)?;

        let cache_key = if preprompt.is_empty() {
            file_hash
        } else {
            format!("{}:{}", file_hash, preprompt)
        };

        if let Some(cached_result) = self.cache.get(&cache_key) {
            return Ok(cached_result);
        }

        let result = self.base.transcribe(audio_path, preprompt)?;

        let _ = self.cache.set(&cache_key, &result);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::traits::{MockCache, MockSttClient};
    use std::fs;
    use std::path::PathBuf;

    // Helper to create a temporary audio file for hash computation
    fn create_temp_audio(content: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("test_audio_{}.wav", uuid::Uuid::new_v4()));
        fs::write(&path, content).unwrap();
        path
    }

    // ============================================================
    // Test 1: File found in cache (base client NOT called)
    // ============================================================
    #[test]
    fn test_cache_hit_returns_cached_result() {
        let path = create_temp_audio("file content");
        let hash = CachedSttClient::compute_file_hash(path.to_str().unwrap()).unwrap();

        let mut mock_cache = MockCache::new();
        mock_cache.expect_get()
            .with(mockall::predicate::eq(hash.clone()))
            .times(1)
            .returning(|_| {
                Some(TranscriptionResult {
                    text: "from cache".to_string(),
                    segments: vec![],
                    language: "en".to_string(),
                    processing_time: 0.1,
                })
            });

        let mut mock_base = MockSttClient::new();
        mock_base.expect_transcribe().times(0);

        let client = CachedSttClient::new(Box::new(mock_base), Box::new(mock_cache));
        let result = client.transcribe(path.to_str().unwrap(), "").unwrap();

        assert_eq!(result.text, "from cache");

        let _ = fs::remove_file(path);
    }

    // ============================================================
    // Test 2: File not in cache — go to network, save to cache
    // ============================================================
    #[test]
    fn test_cache_miss_calls_base_and_sets_cache() {
        let path = create_temp_audio("new file content");
        let hash = CachedSttClient::compute_file_hash(path.to_str().unwrap()).unwrap();

        let mut mock_cache = MockCache::new();
        mock_cache.expect_get()
            .with(mockall::predicate::eq(hash.clone()))
            .times(1)
            .returning(|_| None);

        mock_cache.expect_set()
            .with(
                mockall::predicate::eq(hash.clone()),
                mockall::predicate::always()
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let expected_result = TranscriptionResult {
            text: "from network".to_string(),
            segments: vec![],
            language: "ru".to_string(),
            processing_time: 1.0,
        };

        let mut mock_base = MockSttClient::new();
        mock_base.expect_transcribe()
            .times(1)
            .returning(move |_, _| Ok(expected_result.clone()));

        let client = CachedSttClient::new(Box::new(mock_base), Box::new(mock_cache));
        let result = client.transcribe(path.to_str().unwrap(), "").unwrap();

        assert_eq!(result.text, "from network");

        let _ = fs::remove_file(path);
    }

    // ============================================================
    // Test 3: If file not found — return error before cache
    // ============================================================
    #[test]
    fn test_file_not_found_returns_error_immediately() {
        let mut mock_cache = MockCache::new();
        mock_cache.expect_get().times(0);

        let mut mock_base = MockSttClient::new();
        mock_base.expect_transcribe().times(0);

        let client = CachedSttClient::new(Box::new(mock_base), Box::new(mock_cache));
        let res = client.transcribe("non_existent_file.wav", "");

        assert!(res.is_err());
        match res.unwrap_err() {
            SttError::FileNotFound(_) => (),
            _ => panic!("Expected FileNotFound"),
        }
    }
}
