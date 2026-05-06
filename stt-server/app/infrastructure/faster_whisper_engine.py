"""FasterWhisperEngine — инфраструктурная реализация STTEngine.

⚠️ Это ЕДИНСТВЕННОЕ место в проекте, где импортируется faster_whisper.
Весь остальной код работает с абстракцией STTEngine.

Модель загружается ОДИН РАЗ через load_model() при старте сервера.
Метод transcribe() использует уже загруженную модель.
"""

import time

from faster_whisper import WhisperModel

from app.config import settings
from app.domain.stt_engine import STTEngine
from app.domain.transcription import TranscriptionResult, TranscriptionSegment


class FasterWhisperEngine:
    """Реализация STTEngine на основе Faster-Whisper.

    ⚠️ Не создавай этот объект напрямую в роутерах.
    Используй DI через FastAPI Depends().
    """

    def __init__(self, model: WhisperModel):
        """Принимает уже загруженную модель.

        ⚠️ Модель НЕ загружается здесь — она передаётся извне.
        Загрузка происходит в lifespan (main.py) один раз.
        """
        self._model = model

    def transcribe(
        self,
        audio_path: str,
        language: str | None = None,
        prompt: str | None = None,
        beam_size: int = 5,
        temperature: float = 0.0,
    ) -> TranscriptionResult:
        start_time = time.monotonic()

        segments_gen, info = self._model.transcribe(
            audio_path,
            language=language,
            initial_prompt=prompt,
            beam_size=beam_size,
            temperature=temperature,
        )

        # ⚠️ segments_gen — это генератор. Нужно материализовать его в список.
        # Если не сделать list(), генератор будет потреблён только один раз.
        segments = []
        full_text_parts = []

        for seg in segments_gen:
            segments.append(
                TranscriptionSegment(
                    start=round(seg.start, 3),
                    end=round(seg.end, 3),
                    text=seg.text.strip(),
                )
            )
            full_text_parts.append(seg.text.strip())

        processing_time = round(time.monotonic() - start_time, 3)

        return TranscriptionResult(
            text=" ".join(full_text_parts),
            segments=segments,
            language=info.language,
            processing_time=processing_time,
        )


def load_model() -> WhisperModel:
    """Загрузить модель Whisper.

    ⚠️ Вызывается ОДИН РАЗ в lifespan при старте сервера.
    Никогда не вызывай это из эндпоинта или use case.
    """
    return WhisperModel(
        model_size_or_path=settings.model_size,
        device="cpu",
        compute_type="int8",
    )
