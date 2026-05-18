# stt-server

FastAPI service for speech-to-text transcription using Faster-Whisper.

## Requirements

- Python 3.11+
- `ffmpeg` available in PATH

## Run locally

```bash
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python -m uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
```

Health endpoint:

```bash
curl http://127.0.0.1:8000/health
```

## Run tests

```bash
pip install -r requirements.txt -r requirements-dev.txt
pytest -q
```

## Environment variables

- `DICTOR_MODEL_SIZE` (default: `base`)
- `DICTOR_HOST` (default: `0.0.0.0`)
- `DICTOR_PORT` (default: `8000`)
