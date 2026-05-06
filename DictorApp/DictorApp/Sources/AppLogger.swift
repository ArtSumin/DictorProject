import Foundation
import os

/// Centralized logger for Dictor app with unified subsystem.
/// Use `AppLogger.category("CategoryName")` or predefined categories.
enum AppLogger {
    private static let subsystem = "com.dictor.app"

    static let app = Logger(subsystem: subsystem, category: "App")
    static let delegate = Logger(subsystem: subsystem, category: "AppDelegate")
    static let appState = Logger(subsystem: subsystem, category: "AppState")
    static let permissions = Logger(subsystem: subsystem, category: "Permissions")
    static let recorder = Logger(subsystem: subsystem, category: "Recorder")
    static let settings = Logger(subsystem: subsystem, category: "Settings")
    static let windowManager = Logger(subsystem: subsystem, category: "WindowManager")

    static func category(_ name: String) -> Logger {
        Logger(subsystem: subsystem, category: name)
    }
}
