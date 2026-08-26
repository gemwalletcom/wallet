// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import Store
import StoreTestKit
import Testing

struct NodeServiceTests {
    @Test
    func nodeURLFetchableReturnsSelectedUrl() async throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))

        try await service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.chainNode(region: .asia).node)

        #expect(service.node(for: .ethereum) == Chain.ethereum.chainNode(region: .asia).node.url.asURL)
    }

    @Test
    func nodeURLFetchableReturnsDefaultWhenNotSet() {
        #expect(NodeService.mock().node(for: .ethereum) == Chain.ethereum.defaultBaseUrl)
    }
}
