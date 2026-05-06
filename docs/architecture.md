# Dictor — Архитектура проекта

> Голосовой ввод для macOS и Windows: записал → распознал → вставил текст.

## Обзор

Dictor — кроссплатформенное десктопное приложение, которое превращает речь в текст.
Архитектура построена вокруг **Rust Core (libdictor)** — 
единого ядра с бизнес-логикой, к которому подключаются платформенные обёртки через FFI.

```
┌────────────────┐          ┌─────────────────┐
│  macOS App     │          │  Windows App    │
│  (SwiftUI)     │          │  (WinUI / Qt)   │
│                │          │                 │
│  UI · Audio    │          │  UI · Audio     │
│  Permissions   │          │  Permissions    │
│  Hotkeys       │          │  Hotkeys        │
└───────┬────────┘          └────────┬────────┘
        │ UniFFI                     │ UniFFI / C FFI
        ▼                            ▼
┌──────────────────────────────────────────────┐
│              Rust Core (libdictor)            │
│                                              │
│  HTTP клиент    │  Очередь запросов           │
│  Retry-логика   │  Кеш по хешу файла         │
│  Health monitor │  Валидация аудио            │
│  (push-based)   │  Preprompt → STT сервер    │
│  История (SQLite)│ Экспорт (txt/srt/json)    │
│  UniFFI интерфейс│ Конфигурация (AppConfig)  │
│                                              │
│  Выходы:                                     │
│    libdictor.a   → macOS (static lib)        │
│    libdictor.dll → Windows (dynamic lib)     │
│    dictor-cli    → тестовый бинарник          │
└───────────────────┬──────────────────────────┘
                    │ HTTP POST multipart/form-data
                    ▼
┌──────────────────────────────────────────────┐
│            Python STT Server                  │
│  FastAPI + Faster-Whisper + Docker            │
│  Порт 8000                                   │
└──────────────────────────────────────────────┘
```

## Принципы архитектуры

### 1. Rust Core — центр всего

Вся бизнес-логика живёт в Rust. Платформенные обёртки — тонкие.

```
Rust Core (libdictor):
├── Domain      — модели, трейты (SttClient, HealthCheck, Cache, …)
├── Application — use cases (transcribe, health, cache, queue, retry, validate, export)
├── Infrastructure — HTTP (reqwest), SQLite (rusqlite), файлы
└── FFI         — UniFFI для Swift / C FFI для Windows
```

### 2. Clean Architecture (во всех блоках)

```
Domain  →  Application  →  Infrastructure  →  Interface (FFI / CLI)
```

- **Domain** — чистая бизнес-логика, не зависит ни от чего
- **Application** — use cases, работает через абстракции (trait)
- **Infrastructure** — реализации (HTTP, SQLite, файлы)
- **Interface** — FFI для платформ + CLI для тестирования

### 3. Связь между блоками

| Связь | Механизм | Формат |
|-------|----------|--------|
| Swift ↔ Rust | **UniFFI** (статическая линковка) | Swift-native типы |
| Rust → Swift (events) | **UniFFI Callback Interface** | push-based (AsyncStream) |
| Windows ↔ Rust | **C FFI** (динамическая линковка) | C-типы, строки |
| Rust ↔ Python | **HTTP** | JSON (multipart upload + prompt) |
| CLI (тест) | Прямой вызов Rust API | Rust API |

### 4. Разделение ответственности

| Компонент | Rust Core | Платформа (Swift/WinUI) |
|-----------|:---------:|:-----------------------:|
| HTTP к STT серверу | ✅ | ❌ |
| Очередь запросов | ✅ | ❌ |
| Retry / health check | ✅ | ❌ |
| Кеш по хешу файла | ✅ | ❌ |
| Валидация аудио | ✅ | ❌ |
| История (SQLite) | ✅ | ❌ |
| Экспорт (txt/srt/json) | ✅ | ❌ |
| Конфигурация | ✅ | ❌ |
| Запись аудио | ❌ | ✅ |
| UI | ❌ | ✅ |
| Системные разрешения | ❌ | ✅ |
| Горячие клавиши | ❌ | ✅ |
| Буфер обмена | ❌ | ✅ |

## API контракт (HTTP: Rust ↔ Python)

### `POST /transcribe`

**Request:** `multipart/form-data` с полями `file` и `prompt` (опциональный preprompt)

**Response (200):**
```json
{
  "text": "Распознанный текст целиком",
  "segments": [
    {"start": 0.0, "end": 6.0, "text": "Первый сегмент"},
    {"start": 6.0, "end": 12.0, "text": "Второй сегмент"}
  ],
  "language": "ru",
  "processing_time": 2.195
}
```

### `GET /health`

```json
{"status": "ok", "model_loaded": true}
```

## Этапы разработки

| Этап | Блок | Статус | Тесты |
|:----:|------|:------:|:-----:|
| 1 | Python STT Server | ✅ Готов | 6 |
| 2 | Rust Core (libdictor) | ✅ Готов | 69 |
| 3 | SwiftUI macOS App | 🔧 В работе | — |
| 4 | Windows App | ⏳ Позднее | — |

### Этап 2 — подэтапы:

| # | Задача | Статус |
|---|--------|:------:|
| 2.1 | HTTP клиент + CLI | ✅ |
| 2.2 | Очередь запросов | ✅ |
| 2.3 | Retry-логика | ✅ |
| 2.4 | Health monitor (push-based) | ✅ |
| 2.5 | Кеш по хешу файла | ✅ |
| 2.6 | Валидация аудио | ✅ |
| 2.7 | История (SQLite) | ✅ |
| 2.8 | Экспорт (txt/srt/json) | ✅ |
| 2.9 | UniFFI интерфейс | ✅ |
| 2.10 | Preprompt через всю цепочку | ✅ |
| 2.11 | Сохранение истории при транскрипции | ✅ |

## Стек технологий

| Блок | Язык | Фреймворки | Тесты |
|------|------|------------|-------|
| STT Server | Python 3.11 | FastAPI, Faster-Whisper, Uvicorn | pytest |
| Rust Core | Rust 1.93 | clap, reqwest, serde, thiserror, rusqlite | cargo test, mockall |
| macOS App | Swift | SwiftUI, Tuist, AVAudioRecorder | XCTest / Swift Testing |
| Windows App | TBD | TBD | TBD |
| Инфраструктура | — | Docker, docker-compose | — |
