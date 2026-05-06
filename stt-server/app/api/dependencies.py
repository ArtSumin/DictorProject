"""Dependency Injection для FastAPI.

⚠️ Здесь собираются зависимости для роутеров.
Аналог DI-контейнера (Swinject/Coordinator) в iOS.

get_engine() — единственная точка получения STTEngine.
В тестах подменяется через app.dependency_overrides.
"""

from app.domain.stt_engine import STTEngine


# ⚠️ Хранилище engine. Заполняется в lifespan, читается в роутерах.
# Это НЕ глобальный синглтон — это managed state через DI.
_engine: STTEngine | None = None


def set_engine(engine: STTEngine) -> None:
    """Вызывается один раз в lifespan при старте."""
    global _engine
    _engine = engine


def get_engine() -> STTEngine:
    """FastAPI Depends() вызывает эту функцию для каждого запроса.

    ⚠️ Если engine не инициализирован — это баг, не fallback.
    """
    if _engine is None:
        raise RuntimeError("STTEngine not initialized. Check lifespan.")
    return _engine
