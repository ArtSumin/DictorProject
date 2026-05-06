"""Тесты для абстракции STTEngine.

⚠️ STTEngine — это Protocol (аналог protocol в Swift).
Domain-слой определяет ЧТО нужно, infrastructure реализует КАК.
Тестируем, что абстракция правильно определена и мок её реализует.
"""

from app.domain.stt_engine import STTEngine
from app.domain.transcription import TranscriptionResult, TranscriptionSegment


class MockSTTEngine:
    """Мок-реализация STTEngine для тестов.

    ⚠️ Это НЕ FasterWhisperEngine. Мы никогда не грузим
    реальную модель в unit-тестах.
    """

    def __init__(self, result: TranscriptionResult | None = None):
        self._result = result or TranscriptionResult(
            text="mock transcription",
            segments=[],
            language="en",
            processing_time=0.01,
        )
        self.transcribe_called_with: str | None = None
        self.call_count = 0

    def transcribe(
        self,
        audio_path: str,
        language: str | None = None,
        prompt: str | None = None,
        beam_size: int = 5,
        temperature: float = 0.0,
    ) -> TranscriptionResult:
        self.transcribe_called_with = audio_path
        self.call_count += 1
        return self._result


class TestSTTEngineProtocol:
    """Проверяем, что Protocol определён корректно."""

    def test_mock_implements_protocol(self):
        """MockSTTEngine соответствует протоколу STTEngine."""
        engine: STTEngine = MockSTTEngine()
        assert isinstance(engine, STTEngine)

    def test_transcribe_returns_result(self):
        expected = TranscriptionResult(
            text="Hello",
            segments=[TranscriptionSegment(start=0.0, end=1.0, text="Hello")],
            language="en",
            processing_time=0.1,
        )
        engine: STTEngine = MockSTTEngine(result=expected)
        result = engine.transcribe("test.wav")

        assert result.text == "Hello"
        assert result.language == "en"
        assert len(result.segments) == 1

    def test_transcribe_accepts_optional_params(self):
        engine: STTEngine = MockSTTEngine()
        result = engine.transcribe(
            "test.wav",
            language="ru",
            prompt="подсказка",
            beam_size=3,
            temperature=0.2,
        )
        assert result is not None
