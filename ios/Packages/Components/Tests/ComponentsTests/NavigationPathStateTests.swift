// Copyright (c). Gem Wallet. All rights reserved.

@testable import Components
import Testing

@MainActor
struct NavigationPathStateTests {
    @Test
    func appendDuplicateElement() {
        let state = NavigationPathState()
        state.append(TestScene(id: "a"))

        #expect(state.append(TestScene(id: "a")) == false)
        #expect(state.count == 1)
    }

    @Test
    func appendDifferentElement() {
        let state = NavigationPathState()
        state.append(TestScene(id: "a"))

        #expect(state.append(TestScene(id: "b")) == true)
        #expect(state.count == 2)
    }

    @Test
    func appendDifferentType() {
        let state = NavigationPathState()
        state.append(TestScene(id: "a"))

        #expect(state.append(OtherScene(id: "a")) == true)
        #expect(state.count == 2)
    }

    @Test
    func setPathOverwrites() {
        let state = NavigationPathState()
        state.append(TestScene(id: "x"))

        state.setPath([TestScene(id: "a"), TestScene(id: "b")])

        #expect(state.count == 2)
    }

    @Test
    func reset() {
        let state = NavigationPathState()
        state.append(TestScene(id: "a"))

        state.reset()

        #expect(state.isEmpty == true)
    }
}

private struct TestScene: Hashable, Codable {
    let id: String
}

private struct OtherScene: Hashable, Codable {
    let id: String
}
