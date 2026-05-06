// infrastructure/cache.rs — disk-based cache implementation (File System)
//
// Caches transcription results as JSON files keyed by file hash.

use crate::domain::models::TranscriptionResult;
use crate::domain::traits::{Cache, SttError};
use std::fs;
use std::path::PathBuf;

/// Storage cache implementation that saves results as JSON files.
pub struct FileHashCache {
    /// Directory for cache files
    cache_dir: PathBuf,
}

impl FileHashCache {
    /// Creates cache in the specified directory.
    /// If the directory does not exist — attempts to create it.
    pub fn new(dir: &str) -> Self {
        let path = PathBuf::from(dir);
        let _ = fs::create_dir_all(&path); // ignore error, will be created on first write or fail there
        
        Self { cache_dir: path }
    }

    /// Builds full path to cache file
    fn file_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key))
    }
}

impl Cache for FileHashCache {
    fn get(&self, key: &str) -> Option<TranscriptionResult> {
        let path = self.file_path(key);
        
        // Guard: read file contents
        if let Ok(content) = fs::read_to_string(&path) {
            // Deserialize JSON
            if let Ok(result) = serde_json::from_str(&content) {
                return Some(result);
            }
        }
        None
    }

    fn set(&self, key: &str, result: &TranscriptionResult) -> Result<(), SttError> {
        // Ensure directory exists (might have been removed)
        let _ = fs::create_dir_all(&self.cache_dir);

        let path = self.file_path(key);
        
        // Сериализуем в JSON
        let json = serde_json::to_string(result)
            .map_err(|e| SttError::ParseError(format!("Cache serialization error: {}", e)))?;
            
        // Пишем на диск
        fs::write(&path, json)
            .map_err(|e| SttError::ParseError(format!("Failed to save cache to {:?}: {}", path, e)))?;
            
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Helper for test directory
    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dictor_cache_test_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_file_hash_cache_set_and_get() {
        let dir = temp_dir();
        let cache = FileHashCache::new(dir.to_str().unwrap());

        let result = TranscriptionResult {
            text: "Test cache".to_string(),
            segments: vec![],
            language: "ru".to_string(),
            processing_time: 1.5,
        };

        let key = "some_unique_hash";

        // 1. Сначала кеш пуст
        assert!(cache.get(key).is_none());

        // 2. Сохраняем в кеш
        assert!(cache.set(key, &result).is_ok());

        // 3. Файл реально создан (проверяем инфраструктуру)
        let file_path = cache.file_path(key);
        assert!(file_path.exists());

        // 4. Достаём из кеша — проверяем десериализацию
        let cached = cache.get(key).unwrap();
        assert_eq!(cached.text, "Test cache");
        assert_eq!(cached.language, "ru");

        // Уборка за собой
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_get_invalid_json_returns_none() {
        let dir = temp_dir();
        let cache = FileHashCache::new(dir.to_str().unwrap());
        let key = "invalid_json_test";

        // Симулируем битый файл (например, юзер отредактировал руками)
        fs::write(cache.file_path(key), "not a json").unwrap();

        // Должен тихо вернуть None, а не падать
        assert!(cache.get(key).is_none());

        let _ = fs::remove_dir_all(dir);
    }
}
