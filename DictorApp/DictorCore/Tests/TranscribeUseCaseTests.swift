import Testing
import Foundation
@testable @preconcurrency import DictorCore

struct MockTranscriptionService: TranscriptionService {
    var result: TranscriptionResult?
    var error: Error?

    func transcribe(audioPath: String) async throws -> TranscriptionResult {
        if let error { throw error }
        return result ?? TranscriptionResult(text: "", segments: [], language: "ru", processingTime: 0.0)
    }

    func getHistory() async throws -> [HistoryRecord] {
        []
    }

    func deleteHistory(id: Int64) async throws {}
}

struct TranscribeUseCaseTests {

    @Test func executeSuccess() async throws {
        let mock = MockTranscriptionService(
            result: TranscriptionResult(text: "Test text", segments: [], language: "en", processingTime: 1.0)
        )
        let useCase = TranscribeUseCase(service: mock)

        let tempFile = FileManager.default.temporaryDirectory.appendingPathComponent("test_\(UUID().uuidString).wav")
        FileManager.default.createFile(atPath: tempFile.path, contents: Data())
        defer { try? FileManager.default.removeItem(at: tempFile) }

        let result = try await useCase.transcribe(audioFileURL: tempFile)
        #expect(result.text == "Test text")
    }

    @Test func executeFailure() async {
        let mock = MockTranscriptionService(
            error: NSError(domain: "Test", code: 1, userInfo: [NSLocalizedDescriptionKey: "File not found"])
        )
        let useCase = TranscribeUseCase(service: mock)

        let tempFile = FileManager.default.temporaryDirectory.appendingPathComponent("missing_\(UUID().uuidString).wav")

        do {
            _ = try await useCase.transcribe(audioFileURL: tempFile)
            Issue.record("Expected error but succeeded")
        } catch {
            // Error propagated correctly
        }
    }
}
