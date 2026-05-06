"""Интеграционные тесты API.

⚠️ Тестируем через httpx + ASGITransport.
Faster-Whisper НЕ загружается — мокаем engine.
"""

import pytest
from unittest.mock import MagicMock, patch
from httpx import AsyncClient, ASGITransport

from app.domain.transcription import TranscriptionResult, TranscriptionSegment


@pytest.fixture
def mock_engine():
    """Мок STTEngine для всех тестов API."""
    engine = MagicMock()
    engine.transcribe.return_value = TranscriptionResult(
        text="Test transcription",
        segments=[TranscriptionSegment(start=0.0, end=1.5, text="Test transcription")],
        language="en",
        processing_time=0.1,
    )
    return engine


@pytest.fixture
def app(mock_engine):
    """FastAPI app с подменённым engine.

    ⚠️ Мы подменяем зависимость через app.dependency_overrides.
    Это аналог подмены DI-контейнера в тестах iOS.
    """
    from app.main import app as _app
    from app.api.dependencies import get_engine

    _app.dependency_overrides[get_engine] = lambda: mock_engine
    yield _app
    _app.dependency_overrides.clear()


class TestHealthEndpoint:

    @pytest.mark.asyncio
    async def test_health_returns_ok(self, app):
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://test") as client:
            response = await client.get("/health")

        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"
        assert "model_loaded" in data


class TestTranscribeEndpoint:

    @pytest.mark.asyncio
    async def test_transcribe_with_file(self, app, mock_engine):
        """POST /transcribe с аудио файлом."""
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://test") as client:
            response = await client.post(
                "/transcribe",
                files={"file": ("test.wav", b"fake audio bytes", "audio/wav")},
            )

        assert response.status_code == 200
        data = response.json()
        assert data["text"] == "Test transcription"
        assert data["language"] == "en"
        assert "segments" in data
        assert "processing_time" in data

    @pytest.mark.asyncio
    async def test_transcribe_without_file_returns_422(self, app):
        """Без файла — ошибка валидации."""
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://test") as client:
            response = await client.post("/transcribe")

        assert response.status_code == 422

    @pytest.mark.asyncio
    async def test_transcribe_with_optional_params(self, app, mock_engine):
        """Опциональные параметры прокидываются в engine."""
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://test") as client:
            response = await client.post(
                "/transcribe",
                files={"file": ("test.wav", b"fake audio", "audio/wav")},
                data={
                    "language": "ru",
                    "prompt": "тест",
                    "beam_size": "3",
                    "temperature": "0.2",
                },
            )

        assert response.status_code == 200
        # Проверяем, что engine.transcribe был вызван с правильными параметрами
        call_kwargs = mock_engine.transcribe.call_args
        assert call_kwargs[1].get("language") == "ru" or call_kwargs.kwargs.get("language") == "ru"
