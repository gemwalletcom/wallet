// Copyright (c). Gem Wallet. All rights reserved.

@testable import NodeService
import NodeServiceTestKit
import Primitives
import Store
import StoreTestKit
import Testing

struct NodeServiceTests {
    @Test
    func getNodeSelectedReturnsDefaultWhenNotSet() async throws {
        #expect(try await NodeService.mock().getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.defaultChainNode.node.url)
    }

    @Test
    func switchNode() async throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))

        try await service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.chainNode(region: .asia).node)
        #expect(try await service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.chainNode(region: .asia).node.url)

        try await service.setNodeSelected(chain: .ethereum, node: Chain.ethereum.chainNode(region: .eu).node)
        #expect(try await service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.chainNode(region: .eu).node.url)
    }

    @Test
    func deletingSelectedCustomNodeFallsBackToDefault() async throws {
        let service = NodeService.mock(nodeStore: .mock(db: .mockWithChains([.ethereum])))
        let custom = Node(url: "https://custom.example", status: .active, priority: 0)

        try await service.addNode(chain: .ethereum, url: custom.url)
        try await service.setNodeSelected(chain: .ethereum, node: custom)
        try await service.delete(chain: .ethereum, node: custom)

        #expect(try await service.getNodeSelected(chain: .ethereum).node.url == Chain.ethereum.defaultChainNode.node.url)
        #expect(try await service.nodes(for: .ethereum).contains { $0.node.url == custom.url } == false)
    }

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
