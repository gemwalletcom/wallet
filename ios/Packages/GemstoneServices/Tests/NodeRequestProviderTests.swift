// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitivesTestKit
import GemstoneServices
import Testing

struct NodeRequestProviderTests {
    @Test
    func everyConnectionUsesTheNodeSelectedAtThatMoment() throws {
        let preferences = GemPreferencesStoreMock()
        let provider = NodeRequestProvider(nodeService: GemNodeService.mock(preferences: preferences), chain: .hyperCore)

        try preferences.set(key: "node_hypercore", value: "https://eu.gemnodes.com/hypercore")
        #expect(provider.makeRequest().url?.absoluteString == "wss://eu.gemnodes.com/hypercore/ws")

        try preferences.set(key: "node_hypercore", value: "https://asia.gemnodes.com/hypercore")
        #expect(provider.makeRequest().url?.absoluteString == "wss://asia.gemnodes.com/hypercore/ws")
    }
}
