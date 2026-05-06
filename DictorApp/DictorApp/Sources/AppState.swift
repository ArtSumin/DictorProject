import SwiftUI
import DictorCore


@MainActor
public final class AppState: ObservableObject {
    @Published public var health = ServerHealth(isOk: false, isModelLoaded: false)
    @Published public var coreError: String?

    private var observeTask: Task<Void, Never>?

    public init() {}

    public func startObserving(_ events: AsyncStream<CoreEvent>) {
        observeTask?.cancel()
        observeTask = Task { [weak self] in
            AppLogger.appState.info("Observation started")
            for await event in events {
                switch event {
                case .healthChanged(let h):
                    self?.health = h
                    if h.isOk { self?.coreError = nil }
                    AppLogger.appState.debug("Health updated: isOk=\(h.isOk)")
                case .coreError(let msg):
                    self?.coreError = msg
                    AppLogger.appState.warning("Core error: \(msg)")
                }
            }
            AppLogger.appState.info("Event stream ended")
        }
    }

    deinit {
        observeTask?.cancel()
    }
}
