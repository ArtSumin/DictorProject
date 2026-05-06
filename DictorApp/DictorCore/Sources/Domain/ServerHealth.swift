import Foundation

public struct ServerHealth: Sendable, Equatable {
    public let isOk: Bool
    public let isModelLoaded: Bool

    public init(isOk: Bool, isModelLoaded: Bool) {
        self.isOk = isOk
        self.isModelLoaded = isModelLoaded
    }
}
