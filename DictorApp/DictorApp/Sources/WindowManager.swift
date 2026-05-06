import Cocoa

extension Notification.Name {
    static let recorderWindowDidShow = Notification.Name("recorderWindowDidShow")
}

@MainActor
enum WindowManager {
    private static var openWindowAction: ((String) -> Void)?

    /// Called from a SwiftUI view that has access to @Environment(\.openWindow).
    static func register(_ action: @escaping (String) -> Void) {
        openWindowAction = action
        AppLogger.windowManager.info("Window manager registered")
    }

    /// Positions window at bottom-center of the screen containing the mouse cursor.
    static func positionOnCurrentScreen(window: NSWindow, width: CGFloat = 380, height: CGFloat = 240) {
        let mouseLocation = NSEvent.mouseLocation
        let targetScreen = NSScreen.screens.first { NSMouseInRect(mouseLocation, $0.frame, false) }
            ?? NSScreen.main
            ?? NSScreen.screens.first

        guard let screen = targetScreen else { return }

        let visibleFrame = screen.visibleFrame
        let padding: CGFloat = 20
        let x = visibleFrame.midX - width / 2
        let y = visibleFrame.minY + padding
        window.setFrameOrigin(NSPoint(x: x, y: y))
    }

    /// Opens (or brings to front) a SwiftUI Window scene by its id.
    static func show(id: String) {
        AppLogger.windowManager.info("Showing window: \(id)")
        if let window = NSApp.windows.first(where: {
            $0.identifier?.rawValue.contains(id) == true
        }) {
            if id == "recorder" {
                positionOnCurrentScreen(window: window)
                NotificationCenter.default.post(name: .recorderWindowDidShow, object: nil)
            }
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        openWindowAction?(id)
        NSApp.activate(ignoringOtherApps: true)

        if id == "recorder" {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) {
                if let window = NSApp.windows.first(where: { $0.identifier?.rawValue.contains(id) == true }) {
                    positionOnCurrentScreen(window: window)
                }
            }
        }
    }
}
