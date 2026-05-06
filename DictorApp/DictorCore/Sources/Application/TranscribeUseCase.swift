import Foundation
import AppKit

public final class TranscribeUseCase: Sendable {
    private let service: TranscriptionService

    public init(service: TranscriptionService) {
        self.service = service
    }

    public func transcribe(audioFileURL: URL) async throws -> TranscriptionResult {
        let result = try await service.transcribe(audioPath: audioFileURL.path)

        if audioFileURL.path.hasPrefix(NSTemporaryDirectory()) {
            try? FileManager.default.removeItem(at: audioFileURL)
        }

        await MainActor.run {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(result.text, forType: .string)
        }

        return result
    }

    public func getHistory() async throws -> [HistoryRecord] {
        try await service.getHistory()
    }

    public func deleteHistory(id: Int64) async throws {
        try await service.deleteHistory(id: id)
    }
}
