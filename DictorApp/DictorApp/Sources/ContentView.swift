import SwiftUI
import DictorCore

public struct ContentView: View {
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var viewModel: RecorderViewModel

    @AppStorage("autoHideWindow") private var autoHideWindow = true
    @AppStorage("autoRecordOnLaunch") private var autoRecordOnLaunch = true
    @AppStorage("serverAddress") private var serverAddress = "http://127.0.0.1:8000"

    @Environment(\.dismiss) private var dismiss

    @State private var isHistoryOpen = false
    @State private var isConnectionLabelHovered = false
    @State private var isMicHovered = false
    @State private var miniButtonHovered: String? = nil
    @State private var windowOpacity: Double = 0

    public init() {}

    private var screenState: RecordingScreenState {
        if viewModel.isTranscribing && !viewModel.isRecording {
            return .processing
        } else if viewModel.isRecording {
            return .recording
        }
        return .idle
    }

    /// Кнопка записи заблокирована — сервер offline, запись невозможна
    private var isMicBlockedByOffline: Bool {
        screenState == .idle && !appState.health.isOk
    }

    private let cardWidth: CGFloat = 380
    private let expandedHeight: CGFloat = 240
    private let collapsedHeight: CGFloat = 70

    public var body: some View {
        GlassCardView {
            VStack(spacing: 0) {
                if isHistoryOpen {
                    historyDropdown

                    Rectangle()
                        .fill(Color.white.opacity(0.06))
                        .frame(height: 1)
                }

                widgetBar
            }
        }
        .frame(width: cardWidth)
        .fixedSize(horizontal: false, vertical: true)
        .frame(width: cardWidth, height: expandedHeight, alignment: .bottom)
        .opacity(windowOpacity)
        .animation(.easeOut(duration: 0.3), value: windowOpacity)
        .animation(.easeInOut(duration: 0.3), value: isHistoryOpen)
        .animation(.easeInOut(duration: 0.25), value: screenState)
        .background(WindowAccessor { window in
            guard let window else { return }
            window.styleMask = [.borderless]
            window.level = .floating
            window.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
            window.isOpaque = false
            window.backgroundColor = .clear
            window.isMovableByWindowBackground = true
            window.hasShadow = false
            if let contentView = window.contentView {
                contentView.wantsLayer = true
                contentView.layer?.cornerRadius = 20
                contentView.layer?.masksToBounds = true
            }
            WindowManager.positionOnCurrentScreen(window: window, width: cardWidth, height: expandedHeight)
        })
        .onAppear {
            withAnimation(.easeOut(duration: 0.3)) {
                windowOpacity = 1
            }
            viewModel.loadHistory()
            if autoRecordOnLaunch {
                viewModel.savePreviousApp()
                viewModel.startRecording()
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: .recorderWindowDidShow)) { _ in
            windowOpacity = 0
            DispatchQueue.main.async {
                withAnimation(.easeOut(duration: 0.3)) {
                    windowOpacity = 1
                }
            }
        }
        .onChange(of: viewModel.historyEntries.isEmpty) { _, isEmpty in
            if isEmpty { isHistoryOpen = false }
        }
        .onChange(of: viewModel.isTranscribing) { oldValue, newValue in
            if oldValue && !newValue
                && !viewModel.transcriptionResult.contains("Error")
                && viewModel.transcriptionResult != "Transcription text will appear here..."
            {
                if autoHideWindow {
                    dismissWithFadeOut()
                }
            }
        }
    }

    // MARK: - Main horizontal bar

    private var widgetBar: some View {
        HStack(spacing: 8) {
            miniButton(icon: "xmark") { dismissWithFadeOut() }
            if !viewModel.historyEntries.isEmpty {
                miniButton(icon: "clock.arrow.circlepath") { isHistoryOpen.toggle() }
            }
            miniButton(icon: "gearshape") { WindowManager.show(id: "settings") }

            Spacer().frame(width: 4)

            micButton

            Spacer(minLength: 0)

            stateInfo
                .frame(maxWidth: .infinity, alignment: .center)

            Spacer(minLength: 0)
        }
        .padding(.horizontal, 14)
        .frame(height: collapsedHeight)
    }

    // MARK: - Mic button (tap to start/stop)

    private var micButton: some View {
        Button(action: handleMicTap) {
            ZStack {
                Circle()
                    .fill(micFill)
                    .frame(width: 42, height: 42)
                    .shadow(
                        color: micGlowColor.opacity(isMicHovered ? 0.5 : 0.2),
                        radius: isMicHovered ? 10 : 4
                    )

                micIcon
            }
            .scaleEffect((isMicHovered && !isMicBlockedByOffline && screenState != .processing) ? 1.08 : 1.0)
            .animation(.easeOut(duration: 0.15), value: isMicHovered)
        }
        .buttonStyle(.plain)
        .onHover { isMicHovered = $0 }
        .disabled(screenState == .processing || isMicBlockedByOffline)
    }

    @ViewBuilder
    private var micIcon: some View {
        switch screenState {
        case .idle:
            Image(systemName: isMicBlockedByOffline ? "lock.fill" : "mic.fill")
                .font(.system(size: isMicBlockedByOffline ? 14 : 18))
                .foregroundColor(.white.opacity(isMicBlockedByOffline ? 0.6 : 1))
        case .recording:
            RoundedRectangle(cornerRadius: 2)
                .fill(Color.white)
                .frame(width: 14, height: 14)
        case .processing:
            ProgressView()
                .progressViewStyle(.circular)
                .scaleEffect(0.7)
                .tint(.white.opacity(0.7))
        }
    }

    private var micFill: some ShapeStyle {
        switch screenState {
        case .idle:
            if isMicBlockedByOffline {
                AnyShapeStyle(AppColors.controlDisabled)
            } else {
                AnyShapeStyle(
                    LinearGradient(
                        colors: [AppColors.accentPrimary, AppColors.accentSecondary],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
            }
        case .recording:
            AnyShapeStyle(AppColors.accentDestructive)
        case .processing:
            AnyShapeStyle(Color.white.opacity(0.12))
        }
    }

    private var micGlowColor: Color {
        switch screenState {
        case .idle: isMicBlockedByOffline ? .clear : AppColors.accentPrimary
        case .recording: AppColors.accentDestructive
        case .processing: .clear
        }
    }

    private func dismissWithFadeOut() {
        let duration: Double = 0.25
        withAnimation(.easeIn(duration: duration)) {
            windowOpacity = 0
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            dismiss()
        }
    }

    private func handleMicTap() {
        switch screenState {
        case .idle:
            viewModel.savePreviousApp()
            viewModel.startRecording()
        case .recording:
            viewModel.stopRecordingAndTranscribe()
        case .processing:
            break
        }
    }

    // MARK: - State info (center)

    @ViewBuilder
    private var stateInfo: some View {
        switch screenState {
        case .idle:
            VStack(alignment: .leading, spacing: 3) {
                connectionLabel
            }

        case .recording:
            VStack(alignment: .leading, spacing: 6) {
                Text("Recording...")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(.white.opacity(0.8))

                WaveformView(compact: true, audioLevelDecibels: viewModel.audioLevel)
                    .frame(height: 22)
            }

        case .processing:
            Text("Processing text...")
                .font(.system(size: 13, weight: .medium))
                .foregroundColor(.white.opacity(0.6))
        }
    }

    private var connectionLabel: some View {
        HStack(spacing: 5) {
            Text(isConnectionLabelHovered ? serverAddress : (appState.health.isOk ? "Connected" : "Offline"))
                .font(.system(size: 11))
                .foregroundColor(.white.opacity(0.35))
                .lineLimit(1)
                .truncationMode(.middle)
            Circle()
                .fill(appState.health.isOk ? Color.green : Color.red)
                .frame(width: 5, height: 5)
        }
        .animation(.easeInOut(duration: 0.3), value: isConnectionLabelHovered)
        .onHover { isConnectionLabelHovered = $0 }
    }

    // MARK: - Mini button

    private func miniButton(icon: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: 10, weight: .medium))
                .foregroundColor(.white.opacity(miniButtonHovered == icon ? 0.7 : 0.5))
                .frame(width: 22, height: 22)
                .background(Circle().fill(Color.white.opacity(miniButtonHovered == icon ? 0.1 : 0.06)))
                .scaleEffect(miniButtonHovered == icon ? 1.08 : 1.0)
                .animation(.easeOut(duration: 0.15), value: miniButtonHovered)
        }
        .buttonStyle(.plain)
        .onHover { miniButtonHovered = $0 ? icon : nil }
    }

    // MARK: - History dropdown

    private var historyDropdown: some View {
        HistorySidebarView()
            .transition(.move(edge: .top).combined(with: .opacity))
    }
}

#if DEBUG
#Preview("ContentView Idle") {
    UserDefaults.standard.set(false, forKey: "autoRecordOnLaunch")
    return ContentView()
        .environmentObject(PreviewAppState())
        .environmentObject(RecorderViewModel.preview())
        .frame(width: 380, height: 240)
}

#Preview("ContentView Recording") {
    ContentView()
        .environmentObject(PreviewAppState())
        .environmentObject(RecorderViewModel.preview(isRecording: true))
        .frame(width: 380, height: 240)
}

#Preview("ContentView Processing") {
    ContentView()
        .environmentObject(PreviewAppState())
        .environmentObject(RecorderViewModel.preview(isTranscribing: true, transcriptionResult: "Processing..."))
        .frame(width: 380, height: 240)
}

#Preview("ContentView Offline") {
    UserDefaults.standard.set(false, forKey: "autoRecordOnLaunch")
    return ContentView()
        .environmentObject(PreviewAppState(health: ServerHealth(isOk: false, isModelLoaded: false)))
        .environmentObject(RecorderViewModel.preview())
        .frame(width: 380, height: 240)
}
#endif
