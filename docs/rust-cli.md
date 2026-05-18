# Этап 2 — Rust CLI Core

> CLI-приложение `dictor-cli`: принимает путь к аудиофайлу,
> отправляет на STT сервер, выводит распознанный текст.

## Назначение

Исторически этот модуль использовался как мост между macOS-клиентом и Python STT сервером.
Сейчас в основном приложении используется UniFFI bridge (`DictorCore` ↔ Rust), а CLI остается
удобным инструментом для локальной диагностики и ручной проверки сервера.

## Использование

```bash
# Базовый вызов
dictor-cli ./audio.wav

# С кастомным сервером
dictor-cli ./audio.wav --server http://your-stt-server:8000

# Помощь
dictor-cli --help
```

**Поведение:**
- ✅ Успех → текст в `stdout`, exit code `0`
- ❌ Ошибка → сообщение в `stderr`, exit code `1`
- ❌ Нет аргументов → usage hint, exit code `2`

## Архитектура

```
dictor-cli/
├── Cargo.toml                 ← Зависимости (clap, reqwest, serde, thiserror, mockall)
└── src/
    ├── main.rs                ← Точка входа: DI + запуск (без логики!)
    ├── lib.rs                 ← Экспорт модулей
    │
    ├── domain/                ← Чистая бизнес-логика
    │   ├── mod.rs
    │   ├── models.rs          ← TranscriptionResult, Segment
    │   └── traits.rs          ← SttError (enum), SttClient (trait + automock)
    │
    ├── application/           ← Use Cases
    │   ├── mod.rs
    │   └── transcribe.rs      ← TranscribeUseCase
    │
    ├── infrastructure/        ← Внешние зависимости
    │   ├── mod.rs
    │   └── stt_client.rs      ← HttpSttClient (reqwest + multipart)
    │
    └── interface/             ← CLI адаптер
        ├── mod.rs
        └── cli.rs             ← CliArgs (clap — парсинг аргументов)
```

## Clean Architecture — поток данных

```
CLI аргументы ("audio.wav")
    │
    ▼
CliArgs (interface)       ← парсинг через clap
    │
    ▼
main.rs                   ← DI: создаёт HttpSttClient → TranscribeUseCase
    │
    ▼
TranscribeUseCase         ← execute(audio_path) → делегирует в client
    │
    ▼
SttClient (trait)         ← абстракция (domain не знает про HTTP)
    │
    ▼
HttpSttClient             ← реализация: читает файл, POST multipart, парсит JSON
    │
    ▼
HTTP POST /transcribe     ← reqwest → Python STT Server
    │
    ▼
TranscriptionResult       ← serde::Deserialize → result.text → println! → stdout
```

## Domain модели

### TranscriptionResult

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,              // Полный распознанный текст
    pub segments: Vec<Segment>,    // Сегменты с таймкодами
    pub language: String,          // Язык ("ru", "en")
    pub processing_time: f64,      // Время обработки (секунды)
}
```

### Segment

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Segment {
    pub start: f64,    // Начало сегмента (секунды)
    pub end: f64,      // Конец сегмента (секунды)
    pub text: String,  // Текст сегмента
}
```

### SttError

```rust
#[derive(Error, Debug)]
pub enum SttError {
    FileNotFound(String),    // Файл не найден на диске
    NetworkError(String),    // Сервер недоступен
    ServerError(u16),        // HTTP 4xx / 5xx
    ParseError(String),      // Невалидный JSON ответ
}
```

### SttClient (trait)

```rust
#[cfg_attr(test, mockall::automock)]
pub trait SttClient {
    fn transcribe(&self, audio_path: &str) -> Result<TranscriptionResult, SttError>;
}
```

`#[automock]` генерирует `MockSttClient` для тестов автоматически.

## Dependency Injection

```rust
// main.rs — единственное место, где создаются зависимости
fn main() {
    let args = CliArgs::parse();
    let client = HttpSttClient::new(&args.server_url);              // Infrastructure
    let use_case = TranscribeUseCase::new(Box::new(client));        // Application
    match use_case.execute(&args.audio_file) {                      // Execute
        Ok(result) => println!("{}", result.text),
        Err(err) => { eprintln!("Ошибка: {}", err); process::exit(1); }
    }
}
```

- `Box<dyn SttClient>` — dynamic dispatch (аналог `any SttClient` в Swift)
- `TranscribeUseCase` не знает про `HttpSttClient` — только про trait `SttClient`

## Зависимости (Cargo.toml)

| Крейт | Версия | Назначение | Swift-аналог |
|-------|--------|-----------|--------------|
| `clap` | 4 | Парсинг CLI аргументов | ArgumentParser |
| `reqwest` | 0.12 | HTTP клиент (blocking + multipart) | URLSession |
| `serde` | 1 | Сериализация/десериализация | Codable |
| `serde_json` | 1 | JSON парсинг | JSONDecoder |
| `thiserror` | 2 | Удобные ошибки | LocalizedError |
| `mockall` | 0.13 | Мок-фреймворк (dev) | Mock protocols |

## Тесты

```bash
cd dictor-cli

# Все тесты
cargo test

# Тесты одного слоя
cargo test domain::
cargo test application::
cargo test infrastructure::
cargo test interface::

# Конкретный тест
cargo test test_transcribe_success
```

### Покрытие по слоям

| Слой | Файл | Тесты | Что проверяет |
|------|------|:-----:|-------------|
| Domain | `models.rs` | 3 | Создание, Debug, JSON десериализация |
| Domain | `traits.rs` | 6 | Все 4 типа ошибок, trait через mock (успех + ошибка) |
| Application | `transcribe.rs` | 4 | Успех, FileNotFound, ServerError, NetworkError |
| Infrastructure | `stt_client.rs` | 4 | Создание, FileNotFound, NetworkError, trait conformance |
| Interface | `cli.rs` | 3 | Парсинг аргументов, дефолт, ошибка |
| **Итого** | | **20** | |

### Принцип тестирования

- **Domain + Application:** тестируются через `MockSttClient` (mockall) — **без сервера**
- **Infrastructure:** тестируется через реальные файловые операции и отсутствующий сервер — **без сервера**
- **Interface:** тестируется через `CliArgs::parse_from()` — **без запуска процесса**
- **End-to-end:** ручной тест `dictor-cli ./audio.wav` с запущенным STT сервером

## Rust ↔ Swift: таблица соответствий

Для разработчика, приходящего из Swift:

| Концепция | Swift | Rust |
|-----------|-------|------|
| Структура данных | `struct: Codable` | `#[derive(Deserialize)]` struct |
| Интерфейс | `protocol` | `trait` |
| Enum ошибок | `enum: Error` | `#[derive(Error)]` enum |
| Опциональное | `Optional<T>` / `T?` | `Option<T>` |
| Результат | `throws` / `Result` | `Result<T, E>` |
| Pattern matching | `switch` / `if case` | `match` |
| Динамический dispatch | `any Protocol` | `Box<dyn Trait>` |
| Инициализатор | `init(...)` | `fn new(...)` → `Self` |
| Тесты | `XCTest` + `func test...()` | `#[test] fn test_...()` |
| Запуск тестов | `⌘U` | `cargo test` |

## Ключевые решения

1. **Blocking reqwest** — CLI не нуждается в async; blocking API проще в понимании
2. **`Box<dyn SttClient>`** — DI через trait objects, не generics (проще для начала)
3. **`#[cfg_attr(test, automock)]`** — моки генерируются только при тестировании
4. **`thiserror`** — автоматическая реализация `Display` и `Error` через derive-макрос
5. **exit code 1** — для ошибок, чтобы Swift мог проверить `process.terminationStatus`
6. **Только `text` в stdout** — Swift будет читать `readDataToEndOfFile()` и получит чистый текст
