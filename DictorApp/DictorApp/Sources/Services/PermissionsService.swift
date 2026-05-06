import Cocoa
import AVFoundation

@MainActor
enum PermissionsService {

    private static let accessibilityPromptShownKey = "accessibilityPromptShown"

    // MARK: - Accessibility

    static var isAccessibilityGranted: Bool {
        AXIsProcessTrusted()
    }

    /// Lazily ensures Accessibility permission.
    /// First time: shows the system prompt. After that: opens System Settings.
    static func ensureAccessibility() {
        guard !AXIsProcessTrusted() else { return }

        let alreadyPrompted = UserDefaults.standard.bool(forKey: accessibilityPromptShownKey)

        if !alreadyPrompted {
            AppLogger.permissions.info("First Accessibility prompt")
            let opts = [kAXTrustedCheckOptionPrompt.takeRetainedValue(): true] as CFDictionary
            AXIsProcessTrustedWithOptions(opts)
            UserDefaults.standard.set(true, forKey: accessibilityPromptShownKey)
        } else {
            AppLogger.permissions.info("Accessibility already prompted once, opening System Settings")
            showAccessibilitySettings()
        }
    }

    private static func showAccessibilitySettings() {
        let alert = NSAlert()
        alert.messageText = "Accessibility Permission Required"
        alert.informativeText = """
        Dictor needs Accessibility access to automatically paste transcribed text.

        Open System Settings → Privacy & Security → Accessibility \
        and add Dictor to the list of allowed apps.
        """
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Open Settings")
        alert.addButton(withTitle: "Later")

        if alert.runModal() == .alertFirstButtonReturn {
            if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                NSWorkspace.shared.open(url)
            }
        }
    }

    // MARK: - Microphone

    enum MicrophoneStatus {
        case authorized
        case notDetermined
        case denied
    }

    static var microphoneStatus: MicrophoneStatus {
        switch AVCaptureDevice.authorizationStatus(for: .audio) {
        case .authorized: .authorized
        case .notDetermined: .notDetermined
        default: .denied
        }
    }

    /// Returns `true` if microphone access is available.
    /// - `.authorized` → returns immediately.
    /// - `.notDetermined` → triggers the system prompt, returns result.
    /// - `.denied`/`.restricted` → shows alert directing to System Settings.
    static func ensureMicrophoneAccess() async -> Bool {
        switch microphoneStatus {
        case .authorized:
            return true

        case .notDetermined:
            AppLogger.permissions.info("Requesting microphone permission")
            return await AVCaptureDevice.requestAccess(for: .audio)

        case .denied:
            AppLogger.permissions.warning("Microphone denied, directing to System Settings")
            showMicrophoneSettings()
            return false
        }
    }

    private static func showMicrophoneSettings() {
        let alert = NSAlert()
        alert.messageText = "No Microphone Access"
        alert.informativeText = """
        Dictor cannot record audio without microphone permission.

        Open System Settings → Privacy & Security → Microphone \
        and enable access for Dictor.
        """
        alert.alertStyle = .warning
        alert.addButton(withTitle: "Open Settings")
        alert.addButton(withTitle: "Later")

        if alert.runModal() == .alertFirstButtonReturn {
            if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone") {
                NSWorkspace.shared.open(url)
            }
        }
    }
}
