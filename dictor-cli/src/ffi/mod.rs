// ffi/mod.rs — FFI (Foreign Function Interface) bridge using UniFFI
//
// This module exports the high-level Rust API for use from host applications
// (macOS, etc.) without dealing with C pointers.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::application::validate::ValidatedSttClient;
use crate::application::cache::CachedSttClient;
use crate::application::retry::RetrySttClient;
use crate::application::transcribe::TranscribeUseCase;
use crate::domain::models::{TranscriptionResult, HealthStatus, AppConfig, HistoryRecord};
use crate::domain::traits::{HealthCheck, HistoryStorage};
use crate::infrastructure::stt_client::HttpSttClient;
use crate::infrastructure::cache::FileHashCache;
use crate::infrastructure::audio_validator::LocalAudioValidator;
use crate::infrastructure::history::SqliteHistory;

/// Universal FFI error
#[derive(Debug, uniffi::Error, thiserror::Error)]
pub enum DictorError {
    #[error("Internal Error: {message}")]
    InternalError { message: String },
    
    #[error("Validation Error: {message}")]
    ValidationError { message: String },
    
    #[error("Network Error: {message}")]
    NetworkError { message: String },
}

// ============================================================
// Callback Interface (implemented by host)
// ============================================================

/// Observer for push notifications to the host application.
#[uniffi::export(callback_interface)]
pub trait AppStateObserver: Send + Sync {
    fn on_health_changed(&self, status: HealthStatus);
    fn on_error(&self, message: String);
}

// ============================================================
// DI Container (Global state for FFI)
// ============================================================

/// Main application class, initialized by the host.
/// Holds all UseCases and infrastructure dependencies.
#[derive(uniffi::Object)]
pub struct AppContainer {
    transcribe_use_case: Mutex<TranscribeUseCase>,
    config: Arc<Mutex<AppConfig>>,
    observer: Arc<dyn AppStateObserver>,
    cache_path: String,
    history_storage: SqliteHistory,
    shutdown: Arc<Mutex<bool>>,
}

impl AppContainer {
    fn build_transcribe_use_case(server_address: &str, cache_path: &str) -> TranscribeUseCase {
        let http_client = Box::new(HttpSttClient::new(server_address));
        let retry_client = Box::new(RetrySttClient::new(http_client, 3, 1000));
        let cache = Box::new(FileHashCache::new(cache_path));
        let cached_client = Box::new(CachedSttClient::new(retry_client, cache));
        let validator = Box::new(LocalAudioValidator::new(vec!["wav", "mp3", "m4a"], 500));
        let validated_client = Box::new(ValidatedSttClient::new(validator, cached_client));
        TranscribeUseCase::new(validated_client)
    }

    fn spawn_health_monitor(
        config: Arc<Mutex<AppConfig>>,
        observer: Arc<dyn AppStateObserver>,
        shutdown: Arc<Mutex<bool>>,
    ) {
        thread::spawn(move || {
            let mut last_status: Option<HealthStatus> = None;
            let mut cached_address = String::new();
            let mut client: Option<HttpSttClient> = None;

            loop {
                if *shutdown.lock().unwrap() {
                    break;
                }

                let server_address = config.lock().unwrap().server_address.clone();
                if server_address != cached_address {
                    client = Some(HttpSttClient::new(&server_address));
                    cached_address = server_address;
                }

                if let Some(ref c) = client {
                    let current = match c.check_health() {
                        Ok(status) => status,
                        Err(e) => {
                            observer.on_error(format!("Health check failed: {}", e));
                            HealthStatus {
                                status: "error".to_string(),
                                model_loaded: false,
                            }
                        }
                    };

                    let changed = last_status.as_ref() != Some(&current);
                    if changed {
                        observer.on_health_changed(current.clone());
                        last_status = Some(current);
                    }
                }

                thread::sleep(Duration::from_secs(15));
            }
        });
    }
}

#[uniffi::export]
impl AppContainer {
    /// Core initialization (DI Container Assembly).
    #[uniffi::constructor]
    pub fn new(
        db_path: String,
        cache_path: String,
        config: AppConfig,
        observer: Box<dyn AppStateObserver>,
    ) -> Result<Arc<Self>, DictorError> {
        let observer: Arc<dyn AppStateObserver> = Arc::from(observer);
        let history = SqliteHistory::new(&db_path).map_err(|e| DictorError::InternalError {
            message: format!("Failed to init History Storage: {}", e),
        })?;

        let transcribe_use_case = Self::build_transcribe_use_case(&config.server_address, &cache_path);
        let config = Arc::new(Mutex::new(config));
        let shutdown = Arc::new(Mutex::new(false));

        Self::spawn_health_monitor(
            Arc::clone(&config),
            Arc::clone(&observer),
            Arc::clone(&shutdown),
        );

        Ok(Arc::new(Self {
            transcribe_use_case: Mutex::new(transcribe_use_case),
            config,
            observer,
            cache_path,
            history_storage: history,
            shutdown,
        }))
    }

    /// Update config at runtime (called from Settings).
    /// Recreates HTTP client with new server address.
    pub fn update_config(&self, config: AppConfig) {
        let new_use_case = Self::build_transcribe_use_case(&config.server_address, &self.cache_path);
        *self.transcribe_use_case.lock().unwrap() = new_use_case;
        *self.config.lock().unwrap() = config;
    }

    /// Synchronous audio transcription.
    /// Preprompt is taken from current config (AppConfig).
    /// On success, result is saved to history (best-effort).
    pub fn transcribe(&self, audio_path: String) -> Result<TranscriptionResult, DictorError> {
        let preprompt = self.config.lock().unwrap().preprompt.clone();
        let result = self.transcribe_use_case
            .lock()
            .unwrap()
            .execute(&audio_path, &preprompt)
            .map_err(|e| DictorError::InternalError {
                message: e.to_string(),
            })?;

        if !result.text.trim().is_empty() {
            let record = HistoryRecord {
                id: None,
                audio_path: audio_path.clone(),
                text: result.text.clone(),
                language: result.language.clone(),
                processing_time: result.processing_time,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            };
            let _ = self.history_storage.save(&record);
        }

        Ok(result)
    }

    /// Returns all transcription history records from SQLite, newest first.
    pub fn get_history(&self) -> Result<Vec<HistoryRecord>, DictorError> {
        self.history_storage
            .get_all()
            .map_err(|e| DictorError::InternalError {
                message: format!("Failed to load history: {}", e),
            })
    }

    /// Deletes a history record by its database id.
    pub fn delete_history(&self, id: i64) -> Result<(), DictorError> {
        self.history_storage.delete(id).map_err(|e| DictorError::InternalError {
            message: format!("Failed to delete history: {}", e),
        })
    }
}

impl Drop for AppContainer {
    fn drop(&mut self) {
        *self.shutdown.lock().unwrap() = true;
    }
}
