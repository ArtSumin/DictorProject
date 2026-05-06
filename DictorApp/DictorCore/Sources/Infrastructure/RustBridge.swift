import Foundation
import os

private enum RustBridgeLog {
    static let log = Logger(subsystem: "com.dictor.core", category: "RustBridge")
}

public final class RustBridge: TranscriptionService, @unchecked Sendable {
    public let events: AsyncStream<CoreEvent>
    private let continuation: AsyncStream<CoreEvent>.Continuation
    private let container: Task<AppContainer, Error>

    public init(config: AppConfig) {
        let (stream, continuation) = AsyncStream.makeStream(of: CoreEvent.self)
        self.events = stream
        self.continuation = continuation

        let observer = CallbackBridge(continuation: continuation)

        self.container = Task.detached(priority: .background) {
            let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            let myAppDir = appSupport.appendingPathComponent("Dictor")
            let legacyDir = appSupport.appendingPathComponent("DictorTestApp")

            let fm = FileManager.default
            if fm.fileExists(atPath: legacyDir.path) && !fm.fileExists(atPath: myAppDir.path) {
                try? fm.moveItem(at: legacyDir, to: myAppDir)
            }

            let dbPath = myAppDir.appendingPathComponent("history.sqlite").path
            let cachePath = myAppDir.appendingPathComponent("cache").path

            try? fm.createDirectory(atPath: cachePath, withIntermediateDirectories: true)

            do {
                let container = try AppContainer(
                    dbPath: dbPath,
                    cachePath: cachePath,
                    config: config,
                    observer: observer
                )
                CoreLogger.rustBridge.info("AppContainer created OK")
                return container
            } catch {
                CoreLogger.rustBridge.error("AppContainer init failed: \(error)")
                throw error
            }
        }
    }

    public func transcribe(audioPath: String) async throws -> TranscriptionResult {
        let c = try await container.value
        return try await Task.detached(priority: .utility) { [c] in
            return try c.transcribe(audioPath: audioPath)
        }.value
    }

    public func updateConfig(_ config: AppConfig) async throws {
        let c = try await container.value
        await Task.detached(priority: .utility) { [c] in
            c.updateConfig(config: config)
        }.value
    }

    public func getHistory() async throws -> [HistoryRecord] {
        let c = try await container.value
        return try await Task.detached(priority: .utility) { [c] in
            try c.getHistory()
        }.value
    }

    public func deleteHistory(id: Int64) async throws {
        let c = try await container.value
        try await Task.detached(priority: .utility) { [c] in
            try c.deleteHistory(id: id)
        }.value
    }

    deinit {
        continuation.finish()
    }
}

private final class CallbackBridge: AppStateObserver {
    private let continuation: AsyncStream<CoreEvent>.Continuation

    init(continuation: AsyncStream<CoreEvent>.Continuation) {
        self.continuation = continuation
    }

    func onHealthChanged(status: HealthStatus) {
        CoreLogger.rustBridge.debug("onHealthChanged: status=\(status.status), model=\(status.modelLoaded)")
        let health = ServerHealth(
            isOk: status.status == "ok",
            isModelLoaded: status.modelLoaded
        )
        continuation.yield(.healthChanged(health))
    }

    func onError(message: String) {
        continuation.yield(.coreError(message))
    }
}
