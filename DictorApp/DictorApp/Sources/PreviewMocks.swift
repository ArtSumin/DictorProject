#if DEBUG
import SwiftUI
@preconcurrency import DictorCore

// MARK: - Stub TranscriptionService

private struct StubTranscriptionService: TranscriptionService {
    var result: TranscriptionResult = TranscriptionResult(
        text: "Preview transcription result",
        segments: [],
        language: "ru",
        processingTime: 1.0
    )

    func transcribe(audioPath: String) async throws -> TranscriptionResult {
        result
    }

    func getHistory() async throws -> [HistoryRecord] {
        []
    }

    func deleteHistory(id: Int64) async throws {}
}

// MARK: - Stub AudioRecordingService

private actor StubAudioService: AudioRecordingService {
    func requestPermission() async -> Bool { true }
    func startRecording() async throws {}
    func stopRecording() async throws -> URL {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent("preview_stub_\(UUID().uuidString).wav")
        FileManager.default.createFile(atPath: url.path, contents: Data())
        return url
    }
    func currentLevelDecibels() async -> Float { -20 }
}

// MARK: - PreviewAppState

public final class PreviewAppState: ObservableObject {
    @Published public var health = ServerHealth(isOk: true, isModelLoaded: true)
    @Published public var coreError: String?

    public init(health: ServerHealth = ServerHealth(isOk: true, isModelLoaded: true)) {
        self.health = health
    }

    public func startObserving(_ events: AsyncStream<CoreEvent>) {}
}

// MARK: - RecorderViewModel.preview()

extension RecorderViewModel {
    public static func preview(
        isRecording: Bool = false,
        isTranscribing: Bool = false,
        transcriptionResult: String = "Transcription text will appear here...",
        historyEntries: [HistoryEntry] = [
            HistoryEntry(text: "First history entry example", dbId: 1),
            HistoryEntry(text: "Second transcription example for preview", dbId: 2),
        ]
    ) -> RecorderViewModel {
        let stubService = StubTranscriptionService()
        let useCase = TranscribeUseCase(service: stubService)
        let audioService = StubAudioService()
        let vm = RecorderViewModel(transcribeUseCase: useCase, audioService: audioService)
        vm.isRecording = isRecording
        vm.isTranscribing = isTranscribing
        vm.transcriptionResult = transcriptionResult
        vm.historyEntries = historyEntries
        return vm
    }
}
#endif
