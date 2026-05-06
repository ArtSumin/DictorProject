import Testing
@testable import DictorCore

struct ServerHealthTests {

    @Test func defaultStateIsOffline() {
        let health = ServerHealth(isOk: false, isModelLoaded: false)
        #expect(!health.isOk)
        #expect(!health.isModelLoaded)
    }

    @Test func onlineWithModel() {
        let health = ServerHealth(isOk: true, isModelLoaded: true)
        #expect(health.isOk)
        #expect(health.isModelLoaded)
    }

    @Test func onlineWithoutModel() {
        let health = ServerHealth(isOk: true, isModelLoaded: false)
        #expect(health.isOk)
        #expect(!health.isModelLoaded)
    }

    @Test func equalityWorks() {
        let a = ServerHealth(isOk: true, isModelLoaded: true)
        let b = ServerHealth(isOk: true, isModelLoaded: true)
        let c = ServerHealth(isOk: false, isModelLoaded: true)
        #expect(a == b)
        #expect(a != c)
    }
}
