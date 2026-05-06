"""Тесты для TranscribeUseCase.

⚠️ Use Case тестируется через mock STTEngine.
Faster-Whisper не используется в unit-тестах.
"""

import pytest

from app.domain.transcription import TranscriptionResult, TranscriptionSegment
from app.domain.stt_engine import STTEngine
from app.application.transcribe_use_case import TranscribeUseCase


class MockSTTEngine:
    """Мок для тестирования use case."""

    def __init__(self, result: TranscriptionResult | None = None, error: Exception | None = None):
        self._result = result or TranscriptionResult(
            text="mock",
            segments=[],
            language="en",
            processing_time=0.01,
        )
        self._error = error
        self.transcribe_called_with: dict | None = None

    def transcribe(
        self,
        audio_path: str,
        language: str | None = None,
        prompt: str | None = None,
        beam_size: int = 5,
        temperature: float = 0.0,
    ) -> TranscriptionResult:
        self.transcribe_called_with = {
            "audio_path": audio_path,
            "language": language,
            "prompt": prompt,
            "beam_size": beam_size,
            "temperature": temperature,
        }
        if self._error:
            raise self._error
        return self._result


class TestTranscribeUseCase:
    """Use case — оркестрация. Принимает путь, вызывает engine, возвращает результат."""

    def test_execute_delegates_to_engine(self):
        """Use case должен делегировать вызов engine и вернуть результат."""
        expected = TranscriptionResult(
            text="Привет мир",
            segments=[TranscriptionSegment(start=0.0, end=1.5, text="Привет мир")],
            language="ru",
            processing_time=0.3,
        )
        engine = MockSTTEngine(result=expected)
        use_case = TranscribeUseCase(engine=engine)

        result = use_case.execute(audio_path="/tmp/test.wav")

        assert result.text == "Привет мир"
        assert result.language == "ru"
        assert engine.transcribe_called_with["audio_path"] == "/tmp/test.wav"

    def test_execute_passes_optional_params(self):
        """Опциональные параметры прокидываются в engine."""
        engine = MockSTTEngine()
        use_case = TranscribeUseCase(engine=engine)

        use_case.execute(
            audio_path="/tmp/test.wav",
            language="ru",
            prompt="в контексте IT",
            beam_size=3,
            temperature=0.2,
        )

        assert engine.transcribe_called_with["language"] == "ru"
        assert engine.transcribe_called_with["prompt"] == "в контексте IT"
        assert engine.transcribe_called_with["beam_size"] == 3
        assert engine.transcribe_called_with["temperature"] == 0.2

    def test_execute_propagates_engine_error(self):
        """⚠️ Use case НЕ ловит ошибки engine — пробрасывает наверх.
        Обработка ошибок — ответственность API-слоя.
        """
        engine = MockSTTEngine(error=RuntimeError("Model failed"))
        use_case = TranscribeUseCase(engine=engine)

        with pytest.raises(RuntimeError, match="Model failed"):
            use_case.execute(audio_path="/tmp/bad.wav")
