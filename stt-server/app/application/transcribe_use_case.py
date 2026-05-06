"""TranscribeUseCase — оркестрация транскрипции.

⚠️ Use Case принимает STTEngine через конструктор (DI).
Никакой бизнес-логики в роутере — роутер вызывает только use case.
"""

from app.domain.stt_engine import STTEngine
from app.domain.transcription import TranscriptionResult


class TranscribeUseCase:
    """Выполнить транскрипцию аудиофайла.

    Принимает путь к файлу и опциональные параметры,
    делегирует engine, возвращает результат.
    Ошибки не ловит — пробрасывает наверх.
    """

    def __init__(self, engine: STTEngine):
        self._engine = engine

    def execute(
        self,
        audio_path: str,
        language: str | None = None,
        prompt: str | None = None,
        beam_size: int = 5,
        temperature: float = 0.0,
    ) -> TranscriptionResult:
        return self._engine.transcribe(
            audio_path=audio_path,
            language=language,
            prompt=prompt,
            beam_size=beam_size,
            temperature=temperature,
        )
