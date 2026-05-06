import Testing
import Foundation
@preconcurrency import DictorCore
@testable @preconcurrency import DictorApp

private struct StubTranscriptionService: TranscriptionService {
    var result: TranscriptionResult?
    var error: Error?

    func transcribe(audioPath: String) async throws -> TranscriptionResult {
        if let error { throw error }
        return result ?? TranscriptionResult(text: "", segments: [], language: "ru", processingTime: 0)
    }

    func getHistory() async throws -> [HistoryRecord] {
        []
    }

    func deleteHistory(id: Int64) async throws {}
}

private struct MockAudioService: AudioRecordingService {
    func requestPermission() async -> Bool { true }
    func startRecording() async throws {}
    func stopRecording() async throws -> URL {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent("mock_recording.wav")
        FileManager.default.createFile(atPath: url.path, contents: Data())
        return url
    }
    func currentLevelDecibels() async -> Float { -160 }
}

private struct DeniedAudioService: AudioRecordingService {
    func requestPermission() async -> Bool { false }
    func startRecording() async throws {}
    func stopRecording() async throws -> URL {
        return FileManager.default.temporaryDirectory.appendingPathComponent("none.wav")
    }
    func currentLevelDecibels() async -> Float { -160 }
}

struct RecorderViewModelTests {

    @Test @MainActor func initialState() {
        let stub = StubTranscriptionService(
            result: TranscriptionResult(text: "", segments: [], language: "ru", processingTime: 0)
        )
        let vm = RecorderViewModel(
            transcribeUseCase: TranscribeUseCase(service: stub),
            audioService: MockAudioService()
        )
        #expect(!vm.isRecording)
        #expect(!vm.isTranscribing)
    }

    @Test @MainActor func toggleStartsRecording() async {
        let stub = StubTranscriptionService(
            result: TranscriptionResult(text: "test", segments: [], language: "ru", processingTime: 0.5)
        )
        let vm = RecorderViewModel(
            transcribeUseCase: TranscribeUseCase(service: stub),
            audioService: MockAudioService()
        )

        vm.toggleRecording()
        try? await Task.sleep(nanoseconds: 100_000_000)
        #expect(vm.isRecording)
    }

    @Test @MainActor func micDeniedShowsError() async {
        let stub = StubTranscriptionService(
            result: TranscriptionResult(text: "", segments: [], language: "ru", processingTime: 0)
        )
        let vm = RecorderViewModel(
            transcribeUseCase: TranscribeUseCase(service: stub),
            audioService: DeniedAudioService()
        )

        vm.startRecording()
        try? await Task.sleep(nanoseconds: 100_000_000)
        #expect(!vm.isRecording)
        #expect(vm.transcriptionResult.contains("microphone"))
    }
}
