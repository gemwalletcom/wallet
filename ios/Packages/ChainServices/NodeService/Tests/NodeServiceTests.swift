// Copyright (c). Gem Wallet. All rights reserved.

@testable import NodeService
import NodeServiceTestKit
import Primitives
import Store
import StoreTestKit
import Testing

struct NodeServiceTests {
    @Test
    func getNodeSelectedReturnsDefaultWhenNotSet() {
        #expect(NodeService.mock().getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.defaultChainNode.node.url)
    }

    @Test
    func setNodeSelectedPersistsNode() throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))

        try service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.chainNode(region: .asia).node)

        #expect(service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.chainNode(region: .asia).node.url)
    }

    @Test
    func switchNode() throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))

        try service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.chainNode(region: .asia).node)
        #expect(service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.chainNode(region: .asia).node.url)

        try service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.chainNode(region: .eu).node)
        #expect(service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.chainNode(region: .eu).node.url)
    }

    @Test
    func nodeURLFetchableReturnsSelectedUrl() throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))

        try service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.chainNode(region: .asia).node)

        #expect(service.node(for: .ethereum) == Chain.ethereum.chainNode(region: .asia).node.url.asURL)
    }

    @Test
    func nodeURLFetchableReturnsDefaultWhenNotSet() {
        let service = NodeService.mock()

        #expect(service.node(for: .ethereum) == Chain.ethereum.defaultBaseUrl)
    }
}
