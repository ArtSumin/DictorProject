import Foundation

public enum CoreEvent: Sendable {
    case healthChanged(ServerHealth)
    case coreError(String)
}
