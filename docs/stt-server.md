# Этап 1 — Python STT Server

> Speech-to-Text сервер на FastAPI + Faster-Whisper, запускается в Docker.

## Назначение

Принимает аудиофайл по HTTP, распознаёт речь через Faster-Whisper,
возвращает JSON с текстом, сегментами и языком.

## Архитектура

```
stt-server/
├── Dockerfile                 ← Multi-stage build (builder + runtime)
├── requirements.txt           ← fastapi, uvicorn, faster-whisper, etc.
├── requirements-dev.txt       ← pytest, httpx
├── pytest.ini
└── app/
    ├── main.py                ← Точка входа, lifespan (загрузка модели)
    ├── config.py              ← Settings из env (DICTOR_MODEL_SIZE)
    │
    ├── domain/                ← Чистая бизнес-логика
    │   ├── transcription.py   ← TranscriptionResult, TranscriptionSegment
    │   └── stt_engine.py      ← STTEngine (Protocol — интерфейс)
    │
    ├── application/           ← Use Cases
    │   └── transcribe_use_case.py  ← TranscribeUseCase
    │
    ├── infrastructure/        ← Внешние зависимости
    │   └── faster_whisper_engine.py  ← FasterWhisperEngine + load_model()
    │
    └── api/                   ← HTTP слой
        ├── routes.py          ← POST /transcribe, GET /health
        └── dependencies.py    ← DI: get_engine() / set_engine()
```

## Clean Architecture — поток данных

```
HTTP Request
    │
    ▼
routes.py (API)          ← парсит multipart, сохраняет tmp файл
    │
    ▼
TranscribeUseCase        ← оркестрация (принимает engine через DI)
    │
    ▼
STTEngine (Protocol)     ← абстракция (domain не знает про Whisper)
    │
    ▼
FasterWhisperEngine      ← реализация (infrastructure)
    │
    ▼
Faster-Whisper model     ← нейросеть (загружена один раз при старте)
    │
    ▼
TranscriptionResult      ← домен-модель → JSON → HTTP Response
```

## Domain модели

### TranscriptionResult

```python
class TranscriptionResult(BaseModel):
    text: str                              # Полный распознанный текст
    segments: list[TranscriptionSegment]   # Сегменты с таймкодами
    language: str                          # Распознанный язык ("ru", "en")
    processing_time: float                 # Время обработки в секундах
```

### TranscriptionSegment

```python
class TranscriptionSegment(BaseModel):
    start: float   # Начало сегмента (секунды)
    end: float     # Конец сегмента (секунды)
    text: str      # Текст сегмента
```

### STTEngine (Protocol)

```python
@runtime_checkable
class STTEngine(Protocol):
    def transcribe(
        self,
        audio_path: str,
        language: str | None = None,
        prompt: str | None = None,
        beam_size: int = 5,
        temperature: float = 0.0,
    ) -> TranscriptionResult: ...
```

## Конфигурация

| Переменная | Описание | По умолчанию |
|-----------|----------|:------------:|
| `DICTOR_MODEL_SIZE` | Размер модели Whisper | `base` |
| `DICTOR_HOST` | Хост сервера | `0.0.0.0` |
| `DICTOR_PORT` | Порт сервера | `8000` |
| `HF_TOKEN` | Токен Hugging Face (опционально) | — |

## Загрузка модели

⚠️ **Модель загружается ОДИН РАЗ** при старте сервера в `lifespan`:

```python
@asynccontextmanager
async def lifespan(app: FastAPI):
    model = load_model()                    # ← Один раз
    engine = FasterWhisperEngine(model=model)
    set_engine(engine)                      # ← DI через глобальный holder
    yield
```

Модели кешируются в Docker volume `dictor-models` (`/root/.cache/huggingface`),
чтобы не скачивать при каждом запуске контейнера.

## Docker

### Запуск

```bash
# Сборка и запуск
docker compose up --build

# Только сборка
docker compose build

# В фоне
docker compose up -d
```

### docker-compose.yml

```yaml
services:
  stt-server:
    build:
      context: ./stt-server
    ports:
      - "8000:8000"
    volumes:
      - dictor-models:/root/.cache/huggingface
    environment:
      - DICTOR_MODEL_SIZE=${DICTOR_MODEL_SIZE:-base}
      - HF_TOKEN=${HF_TOKEN:-}
    restart: unless-stopped

volumes:
  dictor-models:
```

### Dockerfile (multi-stage)

- **Stage 1 (builder):** устанавливает Python-зависимости
- **Stage 2 (runtime):** копирует зависимости + код, добавляет `ffmpeg`
- **Healthcheck:** `GET /health` каждые 30 секунд

## API эндпоинты

### `POST /transcribe`

Загрузка и распознавание аудиофайла.

```bash
curl -X POST http://localhost:8000/transcribe \
  -F "file=@audio.wav"
```

**Response (200):**
```json
{
  "text": "Привет, как твои дела",
  "segments": [
    {"start": 0.0, "end": 3.5, "text": "Привет, как твои дела"}
  ],
  "language": "ru",
  "processing_time": 2.195
}
```

### `GET /health`

```bash
curl http://localhost:8000/health
```

**Response (200):**
```json
{"status": "ok", "model_loaded": true}
```

## Тесты

```bash
cd stt-server
pip install -r requirements-dev.txt
pytest -v
```

| Файл | Что тестирует | Кол-во |
|------|--------------|:------:|
| `test_transcription.py` | Domain модели | 1 |
| `test_stt_engine.py` | Protocol conformance | 1 |
| `test_use_case.py` | TranscribeUseCase (mock engine) | 1 |
| `test_routes.py` | API endpoints (TestClient) | 2 |
| `test_config.py` | Settings из env | 1 |
| **Итого** | | **6** |

## Ключевые решения

1. **Protocol, не ABC** — structural typing, легче мокать
2. **Ленивый импорт Whisper** — тесты работают без установки faster-whisper
3. **Временный файл** — Faster-Whisper принимает путь, не байты; файл удаляется в `finally`
4. **Один `load_model()`** — модель загружена в lifespan, переиспользуется для всех запросов
5. **Docker volume для моделей** — модели не скачиваются заново при пересборке образа
