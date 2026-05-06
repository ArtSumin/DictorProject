"""API роутеры.

⚠️ Роутер вызывает ТОЛЬКО use case. Никакой бизнес-логики здесь.
Логика распознавания — в STTEngine.
Оркестрация — в TranscribeUseCase.
Роутер — только HTTP-обёртка.
"""

import tempfile
from pathlib import Path

from fastapi import APIRouter, Depends, File, Form, UploadFile

from app.api.dependencies import get_engine
from app.application.transcribe_use_case import TranscribeUseCase
from app.domain.stt_engine import STTEngine
from app.domain.transcription import TranscriptionResult

router = APIRouter()


@router.get("/health")
async def health():
    """Health check.

    ⚠️ Этот эндпоинт должен работать всегда.
    Rust CLI будет дёргать его чтобы проверить, что сервер жив.
    """
    return {"status": "ok", "model_loaded": True}


@router.post("/transcribe", response_model=TranscriptionResult)
async def transcribe(
    file: UploadFile = File(...),
    language: str | None = Form(None),
    prompt: str | None = Form(None),
    beam_size: int = Form(5),
    temperature: float = Form(0.0),
    engine: STTEngine = Depends(get_engine),
):
    """Транскрибировать аудио файл.

    ⚠️ Файл сохраняется во временную директорию,
    обрабатывается и УДАЛЯЕТСЯ после.
    """
    use_case = TranscribeUseCase(engine=engine)

    # ⚠️ Сохраняем во временный файл — Faster-Whisper принимает путь, не байты.
    suffix = Path(file.filename).suffix if file.filename else ".wav"
    tmp = tempfile.NamedTemporaryFile(delete=False, suffix=suffix)
    try:
        content = await file.read()
        tmp.write(content)
        tmp.flush()
        tmp.close()

        result = use_case.execute(
            audio_path=tmp.name,
            language=language,
            prompt=prompt,
            beam_size=beam_size,
            temperature=temperature,
        )
        return result
    finally:
        # ⚠️ ОБЯЗАТЕЛЬНО удаляем временный файл.
        # Без этого /tmp будет расти бесконечно.
        Path(tmp.name).unlink(missing_ok=True)
