"""Конфигурация STT сервера.

⚠️ model_size фиксирован. Никакой динамической смены модели.
Значение берётся из переменной окружения DICTOR_MODEL_SIZE
и фиксируется при старте сервера.
"""

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    model_size: str = "base"
    host: str = "0.0.0.0"
    port: int = 8000

    model_config = {"env_prefix": "DICTOR_"}


# ⚠️ Синглтон-инстанс. Создаётся один раз при импорте модуля.
# Это единственное "глобальное состояние", которое мы допускаем —
# потому что конфиг иммутабелен после старта.
settings = Settings()
