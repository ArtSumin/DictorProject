import Testing
import DictorCore
@testable @preconcurrency import DictorApp

struct AppStateTests {

    @Test @MainActor func healthChangedUpdatesState() async {
        let (stream, continuation) = AsyncStream.makeStream(of: CoreEvent.self)
        let state = AppState()
        state.startObserving(stream)

        continuation.yield(.healthChanged(ServerHealth(isOk: true, isModelLoaded: true)))
        try? await Task.sleep(nanoseconds: 50_000_000)

        #expect(state.health.isOk)
        #expect(state.health.isModelLoaded)

        continuation.finish()
    }

    @Test @MainActor func coreErrorIsSet() async {
        let (stream, continuation) = AsyncStream.makeStream(of: CoreEvent.self)
        let state = AppState()
        state.startObserving(stream)

        continuation.yield(.coreError("Network timeout"))
        try? await Task.sleep(nanoseconds: 50_000_000)

        #expect(state.coreError == "Network timeout")

        continuation.finish()
    }

    @Test @MainActor func coreErrorClearedOnHealthy() async {
        let (stream, continuation) = AsyncStream.makeStream(of: CoreEvent.self)
        let state = AppState()
        state.startObserving(stream)

        continuation.yield(.coreError("Connection refused"))
        try? await Task.sleep(nanoseconds: 50_000_000)
        #expect(state.coreError != nil)

        continuation.yield(.healthChanged(ServerHealth(isOk: true, isModelLoaded: true)))
        try? await Task.sleep(nanoseconds: 50_000_000)
        #expect(state.coreError == nil)

        continuation.finish()
    }

    @Test @MainActor func coreErrorNotClearedOnUnhealthy() async {
        let (stream, continuation) = AsyncStream.makeStream(of: CoreEvent.self)
        let state = AppState()
        state.startObserving(stream)

        continuation.yield(.coreError("Connection refused"))
        try? await Task.sleep(nanoseconds: 50_000_000)

        continuation.yield(.healthChanged(ServerHealth(isOk: false, isModelLoaded: false)))
        try? await Task.sleep(nanoseconds: 50_000_000)
        #expect(state.coreError == "Connection refused")

        continuation.finish()
    }
}
