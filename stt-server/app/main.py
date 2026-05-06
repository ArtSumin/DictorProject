"""Точка входа FastAPI приложения.

⚠️ lifespan — это главный момент всего сервера.
Модель загружается ОДИН РАЗ здесь. Если ты видишь загрузку модели
где-то ещё — это баг.
"""

from contextlib import asynccontextmanager

from fastapi import FastAPI

from app.api.dependencies import set_engine
from app.api.routes import router


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Startup / Shutdown lifecycle.

    ⚠️ Всё что ДО yield — при старте сервера.
    Всё что ПОСЛЕ yield — при остановке.

    Модель загружается здесь ОДИН РАЗ.
    Никогда не перемещай load_model() в другое место.
    """
    # ⚠️ Ленивый импорт: faster_whisper нужен только при реальном запуске,
    # не при импорте модуля. Это позволяет тестам работать без установки
    # faster-whisper локально — он будет только в Docker.
    from app.infrastructure.faster_whisper_engine import FasterWhisperEngine, load_model

    # Startup: загружаем модель
    model = load_model()
    engine = FasterWhisperEngine(model=model)
    set_engine(engine)

    yield

    # Shutdown: ресурсы освобождаются автоматически


app = FastAPI(
    title="Dictor STT Server",
    description="Speech-to-Text server powered by Faster-Whisper",
    version="0.1.0",
    lifespan=lifespan,
)

app.include_router(router)
