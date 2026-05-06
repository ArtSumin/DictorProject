// infrastructure/audio_validator.rs — File validation implementation

use crate::domain::traits::{AudioValidator, SttError};
use std::fs;
use std::path::Path;

/// Validator for local audio files by extension and size.
pub struct LocalAudioValidator {
    /// Allowed extensions (e.g. ["wav", "mp3", "m4a", "flac"])
    pub allowed_extensions: Vec<String>,
    /// Maximum file size in bytes
    pub max_size_bytes: u64,
}

impl LocalAudioValidator {
    /// Convenience constructor for limit in megabytes (our main limit: 500 MB)
    pub fn new(allowed_extensions: Vec<&str>, max_size_mb: u64) -> Self {
        Self {
            allowed_extensions: allowed_extensions.iter().map(|&s| s.to_string()).collect(),
            max_size_bytes: max_size_mb * 1024 * 1024,
        }
    }
}

impl AudioValidator for LocalAudioValidator {
    fn validate(&self, audio_path: &str) -> Result<(), SttError> {
        let path = Path::new(audio_path);

        // 1. Check existence
        if !path.exists() || !path.is_file() {
            return Err(SttError::FileNotFound(audio_path.to_string()));
        }

        // 2. Check extension
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !self.allowed_extensions.contains(&ext) {
            return Err(SttError::ValidationError(format!(
                "Unsupported file format: .{}. Allowed: {:?}",
                ext, self.allowed_extensions
            )));
        }

        // 3. Check size
        let metadata = fs::metadata(path)
            .map_err(|e| SttError::FileNotFound(format!("{}: {}", audio_path, e)))?;
            
        let size = metadata.len();
        if size > self.max_size_bytes {
            return Err(SttError::ValidationError(format!(
                "File too large: {} MB. Maximum: {} MB",
                size / 1024 / 1024,
                self.max_size_bytes / 1024 / 1024
            )));
        }

        Ok(()) // All good!
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Helper for creating a temp test file
    fn create_temp_file(content: impl AsRef<[u8]>, ext: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("dictor_test_{}.{}", uuid::Uuid::new_v4(), ext));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_valid_file_passes() {
        let validator = LocalAudioValidator::new(vec!["wav", "mp3"], 10);
        let path = create_temp_file(vec![0; 100], "wav"); // 100 байт

        let result = validator.validate(path.to_str().unwrap());
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_invalid_extension_fails() {
        let validator = LocalAudioValidator::new(vec!["wav", "mp3"], 10);
        let path = create_temp_file(vec![0; 100], "txt"); // wrong extension

        let result = validator.validate(path.to_str().unwrap());
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::ValidationError(msg) => {
                assert!(msg.contains("Unsupported file format: .txt"));
            }
            _ => panic!("Expected ValidationError"),
        }

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_too_large_file_fails() {
        // Limit 1 MB only
        let validator = LocalAudioValidator::new(vec!["wav"], 1);
        
        // Create file 1 MB + 10 bytes
        let large_content = vec![0u8; 1024 * 1024 + 10]; 
        let path = create_temp_file(large_content, "wav");

        let result = validator.validate(path.to_str().unwrap());
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::ValidationError(msg) => {
                assert!(msg.contains("File too large: 1 MB. Maximum: 1 MB"));
            }
            _ => panic!("Expected ValidationError"),
        }

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_file_not_found() {
        let validator = LocalAudioValidator::new(vec!["wav"], 10);
        let result = validator.validate("i_do_not_exist.wav");
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::FileNotFound(_) => (),
            _ => panic!("Expected FileNotFound"),
        }
    }
}
