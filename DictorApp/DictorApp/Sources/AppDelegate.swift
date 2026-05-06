import Cocoa
import SwiftUI
import KeyboardShortcuts

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {

    private var statusItem: NSStatusItem!

    var recorderVM: RecorderViewModel?

    func applicationDidFinishLaunching(_ notification: Notification) {
        AppLogger.delegate.info("App launched")
        AppLogger.delegate.info("📂 App path (copy to Finder → Go → Go to Folder): \(Bundle.main.bundlePath)")
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)

        if let button = statusItem.button {
            button.image = NSImage(
                systemSymbolName: "mic.fill",
                accessibilityDescription: "Dictor"
            )
        }

        rebuildMenu()
        setupKeyboardShortcut()
        AppLogger.delegate.info("Menu bar item and shortcuts configured")
    }

    private func rebuildMenu() {
        let menu = NSMenu()

        let recordItem = NSMenuItem(
            title: "Record",
            action: #selector(recordTapped),
            keyEquivalent: "r"
        )
        recordItem.keyEquivalentModifierMask = [.command, .shift]
        menu.addItem(recordItem)

        menu.addItem(.separator())

        let settingsItem = NSMenuItem(
            title: "Settings...",
            action: #selector(settingsTapped),
            keyEquivalent: ","
        )
        settingsItem.keyEquivalentModifierMask = .command
        menu.addItem(settingsItem)

        menu.addItem(.separator())

        let quitItem = NSMenuItem(
            title: "Quit Dictor",
            action: #selector(quitTapped),
            keyEquivalent: "q"
        )
        quitItem.keyEquivalentModifierMask = .command
        menu.addItem(quitItem)

        statusItem.menu = menu
    }

    // MARK: - Actions

    @objc private func recordTapped() {
        AppLogger.delegate.info("Record menu tapped")
        recorderVM?.savePreviousApp()
        WindowManager.show(id: "recorder")
    }

    @objc private func settingsTapped() {
        AppLogger.delegate.info("Settings menu tapped")
        WindowManager.show(id: "settings")
    }

    @objc private func quitTapped() {
        NSApplication.shared.terminate(nil)
    }

    // MARK: - Global keyboard shortcut

    private func setupKeyboardShortcut() {
        KeyboardShortcuts.onKeyUp(for: .toggleRecording) { [weak self] in
            Task { @MainActor in
                guard let self, let vm = self.recorderVM else { return }
                AppLogger.delegate.info("Keyboard shortcut: recording \(vm.isRecording ? "stop" : "start")")
                if vm.isRecording {
                    vm.stopRecordingAndTranscribe()
                } else {
                    vm.savePreviousApp()
                    WindowManager.show(id: "recorder")
                    try? await Task.sleep(for: .milliseconds(200))
                    vm.startRecording()
                }
            }
        }
    }
}
