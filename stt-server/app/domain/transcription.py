"""Domain модели для STT сервера.

⚠️ Эти модели = API-контракт для Rust CLI.
Поля и их типы фиксируются здесь. Меняешь модель —
проверяй, что Rust клиент обновлён.
"""

from pydantic import BaseModel


class TranscriptionSegment(BaseModel):
    """Один сегмент распознанного текста с таймкодами."""
    start: float
    end: float
    text: str


class TranscriptionResult(BaseModel):
    """Полный результат транскрипции.

    ⚠️ Это именно то, что вернёт POST /transcribe.
    Rust CLI будет парсить JSON с этими полями.
    """
    text: str
    segments: list[TranscriptionSegment]
    language: str
    processing_time: float
