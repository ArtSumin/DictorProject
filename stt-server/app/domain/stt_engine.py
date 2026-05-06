"""Абстракция STT движка.

⚠️ Используем typing.Protocol, а не ABC.
Почему Protocol, а не ABC?
- Protocol поддерживает structural typing (утиная типизация).
- Мок-объект не нужно наследовать — достаточно чтобы совпадала сигнатура.
- Это ближе к Swift protocols с structural conformance.

Domain-слой НЕ знает о Faster-Whisper. Он описывает только интерфейс.
"""

from typing import Protocol, runtime_checkable

from app.domain.transcription import TranscriptionResult


@runtime_checkable  # Позволяет isinstance(obj, STTEngine)
class STTEngine(Protocol):
    """Контракт для любого STT движка.

    ⚠️ Единственный движок сейчас — FasterWhisperEngine.
    Но абстракция позволяет легко подменить его в тестах
    или заменить на другой движок в будущем.
    """

    def transcribe(
        self,
        audio_path: str,
        language: str | None = None,
        prompt: str | None = None,
        beam_size: int = 5,
        temperature: float = 0.0,
    ) -> TranscriptionResult:
        """Распознать аудиофайл и вернуть результат."""
        ...
