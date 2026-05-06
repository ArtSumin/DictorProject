// infrastructure/exporter.rs — Transcription formatting implementation

use crate::domain::models::{ExportFormat, TranscriptionResult};
use crate::domain::traits::{Exporter, SttError};

/// Реализация экспорта в различные текстовые форматы.
pub struct FormatExporter;

impl FormatExporter {
    pub fn new() -> Self {
        Self
    }

    /// Преобразует секунды (например 12.500) в формат SRT: HH:MM:SS,mmm
    fn format_srt_time(seconds: f64) -> String {
        let millis = (seconds.fract() * 1000.0).round() as u32;
        let total_secs = seconds.trunc() as u32;
        let sec = total_secs % 60;
        let min = (total_secs / 60) % 60;
        let hour = total_secs / 3600;

        format!("{:02}:{:02}:{:02},{:03}", hour, min, sec, millis)
    }

    /// Генерирует содержимое SRT файла
    fn generate_srt(result: &TranscriptionResult) -> String {
        let mut srt_lines = Vec::new();

        for (i, segment) in result.segments.iter().enumerate() {
            let index = i + 1;
            let start = Self::format_srt_time(segment.start);
            let end = Self::format_srt_time(segment.end);

            srt_lines.push(format!("{}", index));
            srt_lines.push(format!("{} --> {}", start, end));
            srt_lines.push(segment.text.trim().to_string());
            srt_lines.push("".to_string()); // Пустая строка между блоками
        }

        srt_lines.join("\n")
    }
}

impl Exporter for FormatExporter {
    fn export(
        &self,
        result: &TranscriptionResult,
        format: ExportFormat,
    ) -> Result<String, SttError> {
        match format {
            // Обычный текст — просто возвращаем поле text
            ExportFormat::Txt => Ok(result.text.clone()),

            // JSON — сериализуем структуру с отступами (pretty)
            ExportFormat::Json => serde_json::to_string_pretty(result)
                .map_err(|e| SttError::ExportError(format!("JSON serialization failed: {}", e))),

            // SRT — собираем по блокам
            ExportFormat::Srt => Ok(Self::generate_srt(result)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Segment;

    fn get_mock_result() -> TranscriptionResult {
        TranscriptionResult {
            text: "Hello world. This is dictor.".to_string(),
            language: "en".to_string(),
            processing_time: 1.0,
            segments: vec![
                Segment {
                    start: 0.0,
                    end: 1.5,
                    text: "Hello world.".to_string(),
                },
                Segment {
                    start: 1.5,
                    end: 3.25,
                    text: "This is dictor.".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_format_srt_time() {
        assert_eq!(FormatExporter::format_srt_time(0.0), "00:00:00,000");
        assert_eq!(FormatExporter::format_srt_time(1.5), "00:00:01,500");
        assert_eq!(FormatExporter::format_srt_time(61.0), "00:01:01,000");
        assert_eq!(FormatExporter::format_srt_time(3600.999), "01:00:00,999");
    }

    #[test]
    fn test_export_txt() {
        let exporter = FormatExporter::new();
        let result = get_mock_result();
        
        let output = exporter.export(&result, ExportFormat::Txt).unwrap();
        assert_eq!(output, "Hello world. This is dictor.");
    }

    #[test]
    fn test_export_json() {
        let exporter = FormatExporter::new();
        let result = get_mock_result();
        
        let output = exporter.export(&result, ExportFormat::Json).unwrap();
        
        // Просто проверяем, что это валидный JSON и содержит данные
        assert!(output.contains("\"text\": \"Hello world. This is dictor.\""));
        assert!(output.contains("\"start\": 1.5"));
    }

    #[test]
    fn test_export_srt() {
        let exporter = FormatExporter::new();
        let result = get_mock_result();
        
        let output = exporter.export(&result, ExportFormat::Srt).unwrap();
        
        // Проверяем формат SRT
        let expected = "\
1
00:00:00,000 --> 00:00:01,500
Hello world.

2
00:00:01,500 --> 00:00:03,250
This is dictor.
";
        assert_eq!(output.trim(), expected.trim());
    }
}
