// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemNodeService
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import Primitives
import Store
import StoreTestKit
import Testing

struct GemNodeServiceTests {
    @Test
    func nodeURLFetchableReturnsSelectedUrlPerChain() async throws {
        let service = GemNodeService(
            store: GemstoneNodeStore(store: NodeStore.mock(db: .mockWithChains([.ethereum, .solana]))),
            preferences: GemPreferencesStoreMock(),
        )

        try await service.selectNode(chain: Chain.ethereum.rawValue, url: Chain.ethereum.chainNode(region: .asia).node.url)
        try await service.selectNode(chain: Chain.solana.rawValue, url: Chain.solana.chainNode(region: .asia).node.url)
        try await service.selectNode(chain: Chain.ethereum.rawValue, url: Chain.ethereum.chainNode(region: .eu).node.url)

        #expect(service.nodeUrl(chain: Chain.ethereum.rawValue) == Chain.ethereum.chainNode(region: .eu).node.url)
        #expect(service.nodeUrl(chain: Chain.solana.rawValue) == Chain.solana.chainNode(region: .asia).node.url)
    }

    @Test
    func nodeURLFetchableReturnsDefaultWhenNotSet() {
        let service = GemNodeService(store: GemstoneNodeStore(store: NodeStore.mock()), preferences: GemPreferencesStoreMock())

        #expect(service.nodeUrl(chain: Chain.ethereum.rawValue) == Chain.ethereum.defaultBaseUrl.absoluteString)
    }
}
