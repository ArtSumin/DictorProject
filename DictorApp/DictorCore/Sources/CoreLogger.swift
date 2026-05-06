import Foundation
import os

/// Centralized logger for DictorCore framework.
enum CoreLogger {
    private static let subsystem = "com.dictor.core"

    static let rustBridge = Logger(subsystem: subsystem, category: "RustBridge")

    static func category(_ name: String) -> Logger {
        Logger(subsystem: subsystem, category: name)
    }
}
