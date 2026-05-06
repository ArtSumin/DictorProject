import SwiftUI
import DictorCore

@MainActor
final class AppComposition: ObservableObject {
    let bridge: RustBridge
    let appState: AppState
    let recorderVM: RecorderViewModel

    init() {
        let config = AppConfig(
            serverAddress: UserDefaults.standard.string(forKey: "serverAddress") ?? "http://127.0.0.1:8000",
            preprompt: UserDefaults.standard.string(forKey: "preprompt") ?? "Always return the text in the same language as the input."
        )

        let bridge = RustBridge(config: config)
        self.bridge = bridge

        let useCase = TranscribeUseCase(service: bridge)
        let audioService = AVAudioRecorderService()
        self.recorderVM = RecorderViewModel(transcribeUseCase: useCase, audioService: audioService)

        let state = AppState()
        state.startObserving(bridge.events)
        self.appState = state
        AppLogger.app.info("App composition ready")
    }
}

@main
struct DictorApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    @StateObject private var composition = AppComposition()

    var body: some Scene {
        Window("Dictor", id: "recorder") {
            ContentView()
                .environmentObject(composition.appState)
                .environmentObject(composition.recorderVM)
                .wireWindowManager()
                .task { appDelegate.recorderVM = composition.recorderVM }
        }
        .windowResizability(.contentSize)
        .windowStyle(.hiddenTitleBar)

        Window("Dictor Settings", id: "settings") {
            SettingsView(onApplyConfig: { [weak composition] config in
                guard let bridge = composition?.bridge else { return }
                Task {
                    try? await bridge.updateConfig(config)
                }
            })
        }
        .defaultSize(width: 420, height: 520)
        .windowResizability(.contentSize)
        .windowStyle(.hiddenTitleBar)
    }
}

// MARK: - Bridge modifier that captures openWindow for WindowManager

private struct WindowManagerBridge: ViewModifier {
    @Environment(\.openWindow) private var openWindow

    func body(content: Content) -> some View {
        content
            .task {
                WindowManager.register { id in openWindow(id: id) }
            }
    }
}

extension View {
    func wireWindowManager() -> some View {
        modifier(WindowManagerBridge())
    }
}
