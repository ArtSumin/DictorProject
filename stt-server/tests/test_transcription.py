"""Тесты domain-моделей.

⚠️ Domain — фундамент. Эти модели определяют API-контракт
для будущего Rust CLI клиента. Менять их потом будет дорого.
"""

from app.domain.transcription import TranscriptionResult, TranscriptionSegment


class TestTranscriptionSegment:
    """Один сегмент распознанного текста."""

    def test_creation(self):
        segment = TranscriptionSegment(
            start=0.0,
            end=2.5,
            text="Привет мир",
        )
        assert segment.start == 0.0
        assert segment.end == 2.5
        assert segment.text == "Привет мир"

    def test_duration(self):
        segment = TranscriptionSegment(start=1.0, end=3.5, text="test")
        # Длительность сегмента = end - start
        assert segment.end - segment.start == 2.5


class TestTranscriptionResult:
    """Полный результат распознавания."""

    def test_creation_with_segments(self):
        segments = [
            TranscriptionSegment(start=0.0, end=1.0, text="Hello"),
            TranscriptionSegment(start=1.0, end=2.0, text="world"),
        ]
        result = TranscriptionResult(
            text="Hello world",
            segments=segments,
            language="en",
            processing_time=0.5,
        )
        assert result.text == "Hello world"
        assert result.language == "en"
        assert result.processing_time == 0.5
        assert len(result.segments) == 2

    def test_empty_transcription(self):
        """Пустой текст — валидный кейс (тишина в аудио)."""
        result = TranscriptionResult(
            text="",
            segments=[],
            language="ru",
            processing_time=0.1,
        )
        assert result.text == ""
        assert result.segments == []

    def test_serialization_to_dict(self):
        """⚠️ Этот формат — API-контракт для Rust CLI.
        Если поменяешь структуру — сломается клиент.
        """
        result = TranscriptionResult(
            text="Test",
            segments=[TranscriptionSegment(start=0.0, end=1.0, text="Test")],
            language="en",
            processing_time=0.3,
        )
        data = result.model_dump()

        assert "text" in data
        assert "segments" in data
        assert "language" in data
        assert "processing_time" in data
        assert data["segments"][0]["start"] == 0.0
