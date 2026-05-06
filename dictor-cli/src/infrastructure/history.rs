// infrastructure/history.rs — Transcription history repository (SQLite)
//
//
//   class SqliteHistory: HistoryStorage {
//       let dbPath: String
//
//       init(dbPath: String) throws {
//           self.dbPath = dbPath
//           let db = try Connection(dbPath)
//           try db.execute("CREATE TABLE IF NOT EXISTS history ...")
//       }
//
//       func save(record: HistoryRecord) throws {
//           let db = try Connection(dbPath)
//           try db.run("INSERT INTO history ...")
//       }
//
//       func getAll() throws -> [HistoryRecord] { ... }
//   }
//   ```

use crate::domain::models::HistoryRecord;
use crate::domain::traits::{HistoryStorage, SttError};
use rusqlite::{params, Connection};
use std::path::Path;

/// HistoryStorage implementation backed by local SQLite database.
pub struct SqliteHistory {
    /// Path to the database file, provided by the host application
    db_path: String,
}

impl SqliteHistory {
    /// Creates storage and initializes the table if it doesn't exist.
    ///
    /// Path `db_path` must be provided from outside (e.g. host app layer)
    /// so Rust doesn't depend on macOS (Application Support) or Windows-specific paths.
    pub fn new(db_path: &str) -> Result<Self, SttError> {
        // Ensure the database directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| SttError::DatabaseError(format!("Failed to create DB directory: {}", e)))?;
            }
        }

        let repo = Self {
            db_path: db_path.to_string(),
        };

        // Run migration
        repo.init_db()?;

        Ok(repo)
    }

    /// Opens a new connection.
    /// Thread-safe (each call gets its own instance) and very fast in SQLite.
    fn connect(&self) -> Result<Connection, SttError> {
        Connection::open(&self.db_path)
            .map_err(|e| SttError::DatabaseError(format!("Failed to open DB: {}", e)))
    }

    /// Creates the initial table.
    fn init_db(&self) -> Result<(), SttError> {
        let conn = self.connect()?;
        
        // Execute DDL
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                audio_path TEXT NOT NULL,
                text TEXT NOT NULL,
                language TEXT NOT NULL,
                processing_time REAL NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| SttError::DatabaseError(format!("Migration error: {}", e)))?;

        Ok(())
    }
}

impl HistoryStorage for SqliteHistory {
    fn save(&self, record: &HistoryRecord) -> Result<(), SttError> {
        let conn = self.connect()?;

        // Execute INSERT
        conn.execute(
            "INSERT INTO history (audio_path, text, language, processing_time, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                record.audio_path,
                record.text,
                record.language,
                record.processing_time,
                record.created_at,
            ],
        ).map_err(|e| SttError::DatabaseError(format!("Failed to save record: {}", e)))?;

        Ok(())
    }

    fn get_all(&self) -> Result<Vec<HistoryRecord>, SttError> {
        let conn = self.connect()?;

        // Prepare SELECT
        let mut stmt = conn.prepare(
            "SELECT id, audio_path, text, language, processing_time, created_at
             FROM history
             ORDER BY created_at DESC"
        ).map_err(|e| SttError::DatabaseError(format!("Query preparation error: {}", e)))?;

        let record_iter = stmt.query_map([], |row| {
            Ok(HistoryRecord {
                id: Some(row.get(0)?),
                audio_path: row.get(1)?,
                text: row.get(2)?,
                language: row.get(3)?,
                processing_time: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| SttError::DatabaseError(format!("Read error: {}", e)))?;

        let mut results = Vec::new();
        for record in record_iter {
            match record {
                Ok(r) => results.push(r),
                Err(e) => return Err(SttError::DatabaseError(format!("Row parse error: {}", e))),
            }
        }

        Ok(results)
    }

    fn delete(&self, id: i64) -> Result<(), SttError> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|e| SttError::DatabaseError(format!("Failed to delete record: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Генерация уникального пути БД для каждого теста
    fn temp_db_path() -> PathBuf {
        let dir = std::env::temp_dir();
        dir.join(format!("test_dictor_history_{}.sqlite", uuid::Uuid::new_v4()))
    }

    #[test]
    fn test_init_db_creates_table() {
        let db_path = temp_db_path();
        let db_path_str = db_path.to_str().unwrap();

        // Инициализация не должна падать
        let _storage = SqliteHistory::new(db_path_str).expect("Failed to init DB");

        // Убедимся, что файл появился
        assert!(db_path.exists());

        // Cleanup
        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_save_and_get_all() {
        let db_path = temp_db_path();
        let storage = SqliteHistory::new(db_path.to_str().unwrap()).unwrap();

        let record1 = HistoryRecord {
            id: None,
            audio_path: "audio1.wav".to_string(),
            text: "First".to_string(),
            language: "ru".to_string(),
            processing_time: 1.1,
            created_at: 1000,
        };

        let record2 = HistoryRecord {
            id: None,
            audio_path: "audio2.wav".to_string(),
            text: "Second".to_string(),
            language: "en".to_string(),
            processing_time: 2.2,
            created_at: 2000,
        };

        // Сохраняем
        storage.save(&record1).unwrap();
        storage.save(&record2).unwrap();

        // Получаем всё обратно (должна быть сортировка DESC)
        let results = storage.get_all().unwrap();
        
        assert_eq!(results.len(), 2);
        
        // Первый элемент — самый свежий (record2)
        assert_eq!(results[0].text, "Second");
        assert_eq!(results[0].created_at, 2000);
        assert!(results[0].id.is_some()); // SQLite сам выдал ID

        // Второй элемент — старый (record1)
        assert_eq!(results[1].text, "First");
        assert_eq!(results[1].created_at, 1000);

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_empty_database_returns_empty_vec() {
        let db_path = temp_db_path();
        let storage = SqliteHistory::new(db_path.to_str().unwrap()).unwrap();

        let results = storage.get_all().unwrap();
        assert!(results.is_empty());

        let _ = fs::remove_file(db_path);
    }
}
