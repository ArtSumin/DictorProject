import SwiftUI
import KeyboardShortcuts
import DictorCore

public struct SettingsView: View {
    @AppStorage("serverAddress") private var storedServerAddress = "http://127.0.0.1:8000"
    @AppStorage("preprompt") private var storedPreprompt = "Always return the text in the same language as the input."
    @AppStorage("autoHideWindow") private var storedAutoHideWindow = true
    @AppStorage("autoRecordOnLaunch") private var storedAutoRecordOnLaunch = true

    @State private var draftServerAddress: String = ""
    @State private var draftPreprompt: String = ""
    @State private var draftAutoHideWindow: Bool = true
    @State private var draftAutoRecordOnLaunch: Bool = true

    @State private var isCancelHovered = false
    @State private var isApplyHovered = false

    @Environment(\.dismiss) private var dismiss

    var onApplyConfig: ((AppConfig) -> Void)?

    public init(onApplyConfig: ((AppConfig) -> Void)? = nil) {
        self.onApplyConfig = onApplyConfig
    }

    public var body: some View {
        GlassCardView {
            VStack(spacing: 0) {
                settingsHeader
                separator
                settingsBody
                separator
                settingsFooter
            }
        }
        .frame(minWidth: 360, maxWidth: .infinity, minHeight: 400, maxHeight: .infinity)
        .background(WindowAccessor { window in
            guard let window else { return }
            window.styleMask = [.titled, .closable, .fullSizeContentView, .resizable]
            window.titleVisibility = .hidden
            window.titlebarAppearsTransparent = true
            window.isOpaque = false
            window.backgroundColor = .clear
            window.isMovableByWindowBackground = true
            window.hasShadow = true
            if let contentView = window.contentView {
                contentView.wantsLayer = true
                contentView.layer?.cornerRadius = 20
                contentView.layer?.masksToBounds = true
            }
        })
        .onAppear {
            AppLogger.settings.info("Settings opened")
            draftServerAddress = storedServerAddress
            draftPreprompt = storedPreprompt
            draftAutoHideWindow = storedAutoHideWindow
            draftAutoRecordOnLaunch = storedAutoRecordOnLaunch
        }
    }

    // MARK: - Header

    private var settingsHeader: some View {
        HStack(spacing: 6) {
            Image(systemName: "gearshape")
                .font(.system(size: 12))
            Text("Settings")
                .font(.system(size: 13, weight: .medium))
        }
        .foregroundColor(.white.opacity(0.7))
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 20)
        .padding(.vertical, 14)
    }

    // MARK: - Body

    private var settingsBody: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                sectionServerConfig
                sectionPrompts
                sectionBehavior
                sectionShortcuts
            }
            .padding(20)
        }
    }

    // MARK: - Server Config

    private var sectionServerConfig: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionLabel("Server Config")

            darkTextField(text: $draftServerAddress, placeholder: "http://127.0.0.1:8000")
        }
    }

    // MARK: - Preprompt

    private var sectionPrompts: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 4) {
                sectionLabel("Preprompt")
                Text("(e.g. always respond in Russian)")
                    .font(.system(size: 10, weight: .regular))
                    .foregroundColor(.white.opacity(0.35))
            }

            TextEditor(text: $draftPreprompt)
                .font(.system(size: 12))
                .foregroundColor(.white.opacity(0.9))
                .scrollContentBackground(.hidden)
                .padding(8)
                .frame(minHeight: 60, maxHeight: .infinity)
                .background(
                    RoundedRectangle(cornerRadius: 8)
                        .fill(Color.white.opacity(0.05))
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .stroke(Color.white.opacity(0.08), lineWidth: 0.5)
                )
        }
    }

    // MARK: - Behavior

    private var sectionBehavior: some View {
        VStack(alignment: .leading, spacing: 10) {
            sectionLabel("Behavior")

            darkToggle("Start recording when window opens", isOn: $draftAutoRecordOnLaunch)
            darkToggle("Hide window after transcription", isOn: $draftAutoHideWindow)
        }
    }

    // MARK: - Shortcuts

    private var sectionShortcuts: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionLabel("Shortcuts")

            HStack {
                Text("Toggle Recording")
                    .font(.system(size: 12))
                    .foregroundColor(.white.opacity(0.6))
                Spacer()
                KeyboardShortcuts.Recorder("", name: .toggleRecording)
            }
        }
    }

    // MARK: - Footer buttons

    private var settingsFooter: some View {
        HStack(spacing: 12) {
            Spacer()

            Button(action: { dismiss() }) {
                Text("Cancel")
                    .font(.system(size: 12, weight: .medium))
                    .foregroundColor(.white.opacity(0.7))
                    .padding(.horizontal, 20)
                    .padding(.vertical, 8)
                    .background(
                        Capsule()
                            .fill(Color.white.opacity(isCancelHovered ? 0.12 : 0.06))
                    )
                    .animation(.easeOut(duration: 0.12), value: isCancelHovered)
            }
            .buttonStyle(.plain)
            .onHover { isCancelHovered = $0 }
            .keyboardShortcut(.cancelAction)

            Button(action: applySettings) {
                Text("Apply")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundColor(.white)
                    .padding(.horizontal, 20)
                    .padding(.vertical, 8)
                    .background(
                        Capsule()
                            .fill(
                                LinearGradient(
                                    colors: [
                                        AppColors.accentPrimary,
                                        AppColors.accentSecondary,
                                    ],
                                    startPoint: .leading,
                                    endPoint: .trailing
                                )
                            )
                    )
                    .shadow(
                        color: AppColors.accentPrimary.opacity(isApplyHovered ? 0.4 : 0.0),
                        radius: isApplyHovered ? 10 : 0
                    )
                    .scaleEffect(isApplyHovered ? 1.03 : 1.0)
                    .animation(.easeOut(duration: 0.15), value: isApplyHovered)
            }
            .buttonStyle(.plain)
            .onHover { isApplyHovered = $0 }
            .keyboardShortcut(.defaultAction)
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 14)
    }

    // MARK: - Reusable components

    private func sectionLabel(_ title: String) -> some View {
        Text(title)
            .font(.system(size: 11, weight: .semibold))
            .foregroundColor(.white.opacity(0.4))
            .textCase(.uppercase)
            .tracking(0.5)
    }

    private func darkTextField(text: Binding<String>, placeholder: String) -> some View {
        TextField(placeholder, text: text)
            .textFieldStyle(.plain)
            .font(.system(size: 12))
            .foregroundColor(.white.opacity(0.9))
            .padding(10)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color.white.opacity(0.05))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.white.opacity(0.08), lineWidth: 0.5)
            )
    }

    private func darkToggle(_ label: String, isOn: Binding<Bool>) -> some View {
        Toggle(isOn: isOn) {
            Text(label)
                .font(.system(size: 12))
                .foregroundColor(.white.opacity(0.7))
        }
        .toggleStyle(AccentToggleStyle())
    }

    private var separator: some View {
        Rectangle()
            .fill(Color.white.opacity(0.06))
            .frame(height: 1)
    }

    // MARK: - Actions

    private func applySettings() {
        storedServerAddress = draftServerAddress
        storedPreprompt = draftPreprompt
        storedAutoHideWindow = draftAutoHideWindow
        storedAutoRecordOnLaunch = draftAutoRecordOnLaunch

        let config = AppConfig(
            serverAddress: draftServerAddress,
            preprompt: draftPreprompt
        )
        AppLogger.settings.info("Settings applied: server=\(draftServerAddress)")
        onApplyConfig?(config)

        dismiss()
    }

    private static func clearBackgrounds(of view: NSView) {
        if let effectView = view as? NSVisualEffectView {
            effectView.state = .inactive
            effectView.material = .underPageBackground
            effectView.isHidden = true
        }
        view.wantsLayer = true
        view.layer?.backgroundColor = .clear
        view.layer?.isOpaque = false
        for subview in view.subviews {
            clearBackgrounds(of: subview)
        }
    }
}

#if DEBUG
#Preview("SettingsView") {
    SettingsView()
        .frame(width: 420, height: 520)
}
#endif
