// application/export.rs — Use Case: Transcription export

use crate::domain::models::{ExportFormat, TranscriptionResult};
use crate::domain::traits::{Exporter, SttError};
use std::fs;

/// Use Case: Convert and (optionally) save transcription to file.
pub struct ExportUseCase {
    exporter: Box<dyn Exporter>,
}

impl ExportUseCase {
    /// Creates Use Case with injected `Exporter` dependency.
    pub fn new(exporter: Box<dyn Exporter>) -> Self {
        Self { exporter }
    }

    /// Exports the result.
    ///
    /// - `result` — Transcription data.
    /// - `format` — Txt, Srt, or Json.
    /// - `file_path` — If provided, saves the string to this file. If None, just returns the string.
    pub fn execute(
        &self,
        result: &TranscriptionResult,
        format: ExportFormat,
        file_path: Option<&str>,
    ) -> Result<String, SttError> {
        // 1. Delegate formatting to infrastructure layer (via interface)
        let exported_string = self.exporter.export(result, format)?;

        // 2. Optionally save to file
        if let Some(path) = file_path {
            fs::write(path, &exported_string)
                .map_err(|e| SttError::ExportError(format!("Failed to write file: {}", e)))?;
        }

        Ok(exported_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::traits::MockExporter;
    use std::fs;

    fn dummy_result() -> TranscriptionResult {
        TranscriptionResult {
            text: "Hello".to_string(),
            segments: vec![],
            language: "en".to_string(),
            processing_time: 1.0,
        }
    }

    #[test]
    fn test_execute_returns_string_without_saving() {
        let mut mock = MockExporter::new();
        mock.expect_export()
            .times(1)
            .returning(|_, _| Ok("MOCKED EXPORT".to_string()));

        let use_case = ExportUseCase::new(Box::new(mock));
        let result = use_case.execute(&dummy_result(), ExportFormat::Txt, None);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "MOCKED EXPORT");
    }

    #[test]
    fn test_execute_saves_to_file() {
        // Prepare temp file
        let dir = std::env::temp_dir();
        let file_path = dir.join(format!("dictor_export_test_{}.srt", uuid::Uuid::new_v4()));
        let path_str = file_path.to_str().unwrap();

        let mut mock = MockExporter::new();
        mock.expect_export()
            .times(1)
            .returning(|_, _| Ok("1\n00:00:00,000 --> 00:00:01,000\nHello".to_string()));

        let use_case = ExportUseCase::new(Box::new(mock));
        let result = use_case.execute(&dummy_result(), ExportFormat::Srt, Some(path_str));

        assert!(result.is_ok());

        // Verify file was created and contains data
        let saved_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(saved_content, "1\n00:00:00,000 --> 00:00:01,000\nHello");

        // Cleanup
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_execute_propagates_exporter_error() {
        let mut mock = MockExporter::new();
        mock.expect_export()
            .times(1)
            .returning(|_, _| Err(SttError::ExportError("Something broke".to_string())));

        let use_case = ExportUseCase::new(Box::new(mock));
        let result = use_case.execute(&dummy_result(), ExportFormat::Json, None);

        assert!(result.is_err());
        match result.unwrap_err() {
            SttError::ExportError(msg) => assert!(msg.contains("Something broke")),
            _ => panic!("Expected ExportError"),
        }
    }
}
