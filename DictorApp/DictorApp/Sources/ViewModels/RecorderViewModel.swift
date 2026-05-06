import SwiftUI
import DictorCore


@MainActor
public final class RecorderViewModel: ObservableObject {
    @Published public var isRecording = false
    @Published public var isTranscribing = false
    @Published public var transcriptionResult = "Transcription text will appear here..."
    @Published public var historyEntries: [HistoryEntry] = []
    /// dBFS (-160…0). Updated during recording via polling.
    @Published public var audioLevel: Float = -160

    private let transcribeUseCase: TranscribeUseCase
    private let audioService: AudioRecordingService
    private var previousApp: NSRunningApplication?
    private var levelPollingTask: Task<Void, Never>?

    public init(transcribeUseCase: TranscribeUseCase, audioService: AudioRecordingService) {
        self.transcribeUseCase = transcribeUseCase
        self.audioService = audioService
    }

    /// Loads history from persistent storage. Call on app launch or when recorder window appears.
    public func loadHistory() {
        Task(priority: .utility) {
            do {
                let records = try await transcribeUseCase.getHistory()
                let entries = records.reversed().map { record in
                    HistoryEntry(
                        id: UUID(),
                        text: record.text,
                        date: Date(timeIntervalSince1970: TimeInterval(record.createdAt)),
                        dbId: record.id
                    )
                }
                await MainActor.run {
                    self.historyEntries = entries
                }
            } catch {
                AppLogger.recorder.error("Failed to load history: \(error.localizedDescription)")
            }
        }
    }

    /// Deletes a history entry by its database id. Removes from UI and persistent storage.
    public func deleteHistoryEntry(dbId: Int64) {
        Task(priority: .utility) {
            do {
                try await transcribeUseCase.deleteHistory(id: dbId)
                await MainActor.run {
                    withAnimation(.easeInOut(duration: 0.25)) {
                        self.historyEntries.removeAll { $0.dbId == dbId }
                    }
                }
            } catch {
                AppLogger.recorder.error("Failed to delete history: \(error.localizedDescription)")
            }
        }
    }

    public func savePreviousApp() {
        let frontmost = NSWorkspace.shared.frontmostApplication
        if frontmost?.bundleIdentifier != Bundle.main.bundleIdentifier {
            previousApp = frontmost
            AppLogger.recorder.debug("Saved previous app: \(frontmost?.localizedName ?? "nil")")
        }
    }

    public func toggleRecording() {
        if isRecording {
            stopRecordingAndTranscribe()
        } else {
            startRecording()
        }
    }

    public func startRecording() {
        guard !isRecording && !isTranscribing else { return }

        Task(priority: .userInitiated) {
            let granted = await PermissionsService.ensureMicrophoneAccess()
            guard granted else {
                self.transcriptionResult = "Error: No microphone access."
                return
            }

            do {
                try await audioService.startRecording()
                self.isRecording = true
                self.transcriptionResult = "Recording voice..."
                AppLogger.recorder.info("Recording started")
                startLevelPolling()
            } catch {
                self.transcriptionResult = "Recording error: \(error.localizedDescription)"
                AppLogger.recorder.error("Recording failed: \(error.localizedDescription)")
            }
        }
    }

    public func stopRecordingAndTranscribe() {
        guard isRecording else { return }

        self.isRecording = false
        stopLevelPolling()
        self.audioLevel = -160
        self.isTranscribing = true
        self.transcriptionResult = "Recording stopped. Transcribing..."

        Task(priority: .userInitiated) {
            do {
                let url = try await audioService.stopRecording()
                AppLogger.recorder.info("Transcription started for \(url.lastPathComponent)")
                let result = try await transcribeUseCase.transcribe(audioFileURL: url)
                self.transcriptionResult = result.text
                if !result.text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                    self.loadHistory()
                }
                AppLogger.recorder.info("Transcription completed, \(result.text.count) chars")
                pasteIntoPreviousApp()
            } catch {
                self.transcriptionResult = "Transcription error: \(error.localizedDescription)"
                AppLogger.recorder.error("Transcription failed: \(error.localizedDescription)")
            }
            self.isTranscribing = false
        }
    }

    private func pasteIntoPreviousApp() {
        guard let app = previousApp else {
            AppLogger.recorder.info("No previous app to paste into")
            return
        }
        guard PermissionsService.isAccessibilityGranted else {
            AppLogger.recorder.warning("Accessibility not granted, skipping auto-paste")
            PermissionsService.ensureAccessibility()
            return
        }
        AppLogger.recorder.info("Activating previous app: \(app.localizedName ?? "?")")
        app.activate()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
            Self.simulatePaste()
        }
    }

    private static func simulatePaste() {
        let src = CGEventSource(stateID: .combinedSessionState)
        let keyDown = CGEvent(keyboardEventSource: src, virtualKey: 0x09, keyDown: true)
        let keyUp = CGEvent(keyboardEventSource: src, virtualKey: 0x09, keyDown: false)
        keyDown?.flags = .maskCommand
        keyUp?.flags = .maskCommand
        keyDown?.post(tap: .cghidEventTap)
        keyUp?.post(tap: .cghidEventTap)
    }

    private func startLevelPolling() {
        levelPollingTask?.cancel()
        levelPollingTask = Task { [weak self] in
            guard let self else { return }
            while !Task.isCancelled && self.isRecording {
                let level = await self.audioService.currentLevelDecibels()
                self.audioLevel = level
                try? await Task.sleep(nanoseconds: 50_000_000)
            }
            if !self.isRecording {
                self.audioLevel = -160
            }
        }
    }

    private func stopLevelPolling() {
        levelPollingTask?.cancel()
        levelPollingTask = nil
    }

    public func transcribeFile(url: URL) {
        self.isTranscribing = true
        self.transcriptionResult = "Sending file..."
        AppLogger.recorder.info("File transcription started: \(url.lastPathComponent)")

        Task(priority: .utility) {
            do {
                let result = try await transcribeUseCase.transcribe(audioFileURL: url)
                self.transcriptionResult = result.text
                if !result.text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                    self.loadHistory()
                }
                AppLogger.recorder.info("File transcription completed, \(result.text.count) chars")
            } catch {
                self.transcriptionResult = "Error: \(error.localizedDescription)"
                AppLogger.recorder.error("File transcription failed: \(error.localizedDescription)")
            }
            self.isTranscribing = false
        }
    }
}
