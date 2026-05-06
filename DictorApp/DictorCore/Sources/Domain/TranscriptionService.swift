import Foundation

/// Protocol for transcription service.
/// Infrastructure implements via Rust FFI (AppContainer).
/// Tests mock this protocol.
public protocol TranscriptionService: Sendable {
    /// Transcribes audio file at the given path.
    func transcribe(audioPath: String) async throws -> TranscriptionResult

    /// Returns all transcription history records from persistent storage.
    func getHistory() async throws -> [HistoryRecord]

    /// Deletes a history record by its database id.
    func deleteHistory(id: Int64) async throws
}
