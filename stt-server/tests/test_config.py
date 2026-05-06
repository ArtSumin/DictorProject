"""Тесты конфигурации."""

import os
from unittest.mock import patch

from app.config import Settings


class TestSettings:

    def test_default_values(self):
        """Дефолтные значения без переменных окружения."""
        settings = Settings()
        assert settings.model_size == "base"
        assert settings.host == "0.0.0.0"
        assert settings.port == 8000

    def test_model_size_from_env(self):
        """⚠️ model_size читается из env, но НЕ меняется динамически.
        Значение фиксируется при старте и не меняется до рестарта.
        """
        with patch.dict(os.environ, {"DICTOR_MODEL_SIZE": "small"}):
            settings = Settings()
            assert settings.model_size == "small"

    def test_port_from_env(self):
        with patch.dict(os.environ, {"DICTOR_PORT": "9000"}):
            settings = Settings()
            assert settings.port == 9000
