# Dictor

macOS menu bar app for voice recording and speech-to-text transcription. Record audio, get text, and paste it into the active app (notes, chat, etc.).

## Features

- Record voice from the microphone
- Transcription via an external STT server (Whisper, etc.)
- Auto-paste result into the previous app (⌘V)
- Hotkey for recording (⌘⇧R by default)
- Transcription history
- Configurable server address and prompts

## Requirements

- macOS 14.0+
- [Rust](https://rustup.rs/) — to build the Rust library
- [Tuist](https://tuist.io) — to generate the Xcode project (or mise)
- Local STT server (e.g. from `stt-server/` in this repo)

## Installation

### 1. Build Rust library

```bash
./scripts/build-libdictor.sh --bindings
```

For universal binary (arm64 + x86_64) and bindings:

```bash
./scripts/build-libdictor.sh --universal --bindings
```

### 2. Generate Xcode project

```bash
cd DictorApp
tuist generate
# or: mise exec -- tuist generate
```

### 3. Configure environment

Copy the example environment file and edit it as needed:

```bash
cp .env.example .env
```

Available variables:

- `DICTOR_MODEL_SIZE` — Whisper model size (`tiny`, `base`, `small`, `medium`, `large-v3`).
- `HF_TOKEN` — optional Hugging Face token for gated models.

### 4. Run STT server (Python)

```bash
cd stt-server
pip install -r requirements.txt
python -m uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
```

Default server: `http://127.0.0.1:8000`.

Or via Docker Compose from the repo root:

```bash
docker compose up --build
```

### 5. Build and run the app

1. Open `DictorApp/Dictor.xcworkspace` in Xcode.
2. Select scheme **DictorApp** and device **My Mac**.
3. Run (⌘R).

### 6. Permissions

- **Microphone** — for voice recording.
- **Accessibility** — for auto-pasting text into the active app (Settings → Privacy & Security → Accessibility).

## Project structure

```
Dictor/
├── DictorApp/          # macOS SwiftUI app (Tuist)
├── dictor-cli/         # Rust: STT client, FFI, CLI
├── stt-server/         # Python: HTTP STT server (Faster Whisper)
├── scripts/            # build-libdictor.sh
└── ARCHITECTURE.md     # Architecture overview
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for architecture details.

## License

Released under the [MIT License](LICENSE).
