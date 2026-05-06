# Dictor Architecture

Overview of the project structure and component relationships.

---

## Overview

Dictor is a multi-language project: macOS client in Swift, transcription logic in Rust, HTTP STT server in Python. Swift ↔ Rust communication is implemented via UniFFI.

```
┌─────────────────────────────────────────────────────────────────┐
│  DictorApp (SwiftUI, macOS)                                      │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────────┐ │
│  │ Views       │  │ ViewModels   │  │ AppState, Permissions    │ │
│  └──────┬──────┘  └──────┬───────┘  └───────────┬─────────────┘ │
│         │                 │                       │               │
│         └─────────────────┴───────────────────────┘               │
│                           │                                       │
│  ┌────────────────────────▼──────────────────────────────────┐  │
│  │ DictorCore (Framework)                                      │  │
│  │  TranscribeUseCase ← TranscriptionService (RustBridge)      │  │
│  └────────────────────────┬──────────────────────────────────┘  │
└───────────────────────────┼──────────────────────────────────────┘
                            │ UniFFI
┌───────────────────────────▼──────────────────────────────────────┐
│  dictor-cli (Rust lib)                                             │
│  domain → application → infrastructure                             │
│  HttpSttClient → STT Server (HTTP)                                 │
└───────────────────────────┬───────────────────────────────────────┘
                            │ HTTP
┌───────────────────────────▼───────────────────────────────────────┐
│  stt-server (Python)                                                │
│  Faster Whisper, FastAPI                                            │
└────────────────────────────────────────────────────────────────────┘
```

---

## 1. DictorApp (Swift, macOS)

Client app: menu bar, windows, UI.

### Targets

| Target | Purpose |
|--------|---------|
| **DictorApp** | macOS app (SwiftUI, menu bar, windows) |
| **DictorCore** | Framework with business logic and FFI bridge to Rust |
| **DictorAppTests** | App tests |
| **DictorCoreTests** | Core tests |

### DictorApp structure

```
DictorApp/Sources/
├── DictorApp.swift       # Entry point, composition, windows
├── AppDelegate.swift     # Menu bar, hotkeys
├── ContentView.swift     # Main UI (recording, history)
├── Views/                # RecordingView, IdleView, ProcessingView, HistorySidebarView
├── Components/           # GlassCardView, WaveformView, TopBarView
├── ViewModels/           # RecorderViewModel
├── Services/             # AudioRecordingService, PermissionsService
├── AppState.swift        # Server health, errors
├── AppLogger.swift       # Centralized logger
├── WindowManager.swift   # Window management (recorder/settings)
└── SettingsView.swift    # Settings (server, prompts)
```

### Data flow

- **RecorderViewModel** — recording, transcription, history, auto-paste.
- **AppState** — subscribes to RustBridge events (server health, errors).
- **AppComposition** — creates RustBridge, TranscribeUseCase, AppState, RecorderViewModel.

---

## 2. DictorCore (Swift Framework)

Layer between UI and Rust: domain logic and adapters.

```
DictorCore/Sources/
├── Domain/               # TranscriptionService, CoreEvent, ServerHealth
├── Application/          # TranscribeUseCase
├── Infrastructure/       # RustBridge (bridge to libdictor)
├── CoreLogger.swift
└── Dependencies/DictorFFI/
    ├── dictor.swift      # Generated UniFFI bindings
    ├── dictorFFI.h      # C headers
    ├── libdictor.a      # Static Rust library
    └── module.modulemap
```

### RustBridge

- Initializes `AppContainer` from Rust (history, cache, config).
- Calls `transcribe(audioPath:)` and `updateConfig(_:)`.
- Subscribes to events via `AppStateObserver`: health, errors.
- Exposes `AsyncStream<CoreEvent>` for the UI.

---

## 3. dictor-cli (Rust)

Library for STT: HTTP client, validation, cache, retry, history.

### Layers (Clean Architecture)

```
dictor-cli/src/
├── domain/          # Models (TranscriptionResult, AppConfig, HealthStatus)
│   ├── models.rs
│   └── traits.rs    # HealthCheck, HistoryStorage
├── application/     # Use cases
│   ├── transcribe.rs   # TranscribeUseCase
│   ├── cache.rs       # CachedSttClient
│   ├── retry.rs       # RetrySttClient
│   ├── validate.rs    # ValidatedSttClient
│   └── health.rs      # HealthCheck use case
├── infrastructure/   # Implementations
│   ├── stt_client.rs    # HttpSttClient → STT API
│   ├── cache.rs         # FileHashCache
│   ├── audio_validator.rs
│   └── history.rs       # SqliteHistory
├── ffi/              # UniFFI bridge for Swift
│   └── mod.rs        # AppContainer, AppStateObserver, transcribe, update_config
└── interface/       # CLI (optional)
```

### FFI (ffi/mod.rs)

- **AppContainer** — DI container: builds UseCase chain, SqliteHistory, background health monitoring.
- **AppStateObserver** — callback interface for Swift: `on_health_changed`, `on_error`.
- **Methods**: `new`, `transcribe`, `update_config`.

### Build

- `cargo build --release` → `libdictor.a`
- `uniffi-bindgen` → `dictor.swift`, `dictorFFI.h`
- Script `scripts/build-libdictor.sh` — builds and copies to `DictorFFI/`.

---

## 4. stt-server (Python)

HTTP API for transcription, powered by Faster Whisper.

```
stt-server/app/
├── api/           # FastAPI routes, dependencies
├── domain/        # SttEngine, Transcription
├── application/   # TranscribeUseCase
├── infrastructure/# FasterWhisperEngine
└── main.py
```

Protocol: POST `/transcribe` with audio (multipart/form-data), response — JSON with `text`, `segments`, `language`, `processing_time`.

---

## 5. Dependencies

```
DictorApp
  └── DictorCore
        └── libdictor.a (Rust)
        └── dictor.swift (UniFFI)
  └── KeyboardShortcuts (SPM)

dictor-cli
  └── reqwest (HTTP)
  └── rusqlite (history)
  └── uniffi (FFI)
```

---

## 6. Typical transcription flow

1. **User** taps record or ⌘⇧R.
2. **RecorderViewModel** → `AudioRecordingService` records WAV.
3. **RecorderViewModel** → `TranscribeUseCase.transcribe(audioFileURL:)`.
4. **TranscribeUseCase** → `RustBridge.transcribe(audioPath:)`.
5. **RustBridge** → `AppContainer.transcribe()` (Rust).
6. **Rust** → `HttpSttClient` sends POST to STT server.
7. **stt-server** → Faster Whisper, returns JSON.
8. **Rust** → serializes result, saves history, returns to Swift.
9. **RecorderViewModel** → updates UI, with Accessibility — pastes text (⌘V) into the previous app.

---

## 7. Logging

- **AppLogger** (DictorApp): subsystem `com.dictor.app`, categories per component.
- **CoreLogger** (DictorCore): subsystem `com.dictor.core`, category `RustBridge`.

Filter logs in Console.app by subsystem.
