// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNodeServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public final class NodeService: Sendable {
    public let nodeStore: NodeStore
    private let service: any GemNodeServiceProtocol

    public init(
        nodeStore: NodeStore,
        service: any GemNodeServiceProtocol,
    ) {
        self.nodeStore = nodeStore
        self.service = service
    }

    public func defaultNodes(chain: Chain) throws -> [ChainNode] {
        try service.getDefaultNodes(chain: chain.rawValue).map { try ChainNode(chain: chain.rawValue, node: Primitives.Node($0)) }
    }

    public func getNodeSelected(chain: Chain) throws -> ChainNode {
        let node = try service.selectedNode(
            chain: chain.rawValue,
            selectedUrl: nodeStore.selectedNodeUrl(chain: chain),
            storedNodes: nodeStore.nodes(chain: chain).map { try $0.node.json() },
        )
        return try ChainNode(chain: chain.rawValue, node: Primitives.Node(node))
    }

    public func setNodeSelected(chain: Chain, node: Primitives.Node) async throws {
        try await service.setSelectedNode(chain: chain.rawValue, url: node.url)
    }

    public func addNode(chain: Chain, url: String) async throws {
        try await service.addNode(chain: chain.rawValue, url: url)
    }

    public func delete(chain: Chain, node: Primitives.Node) async throws {
        try await service.deleteNode(chain: chain.rawValue, url: node.url)
    }

    public func canDelete(chain: Chain, url: String) -> Bool {
        service.canDeleteNode(chain: chain.rawValue, url: url)
    }

    public func nodes(for chain: Chain) async throws -> [ChainNode] {
        try service.sortedNodes(chain: chain.rawValue, nodes: await service.getNodes(chain: chain.rawValue))
            .map { try ChainNode(chain: chain.rawValue, node: Primitives.Node($0)) }
    }
}

// MARK: - NodeURLProvidable

extension NodeService: NodeURLProvidable {
    public func node(for chain: Chain) -> URL {
        let selectedUrl = try? nodeStore.selectedNodeUrl(chain: chain)
        let storedNodes = (try? nodeStore.nodes(chain: chain).map { try $0.node.json() }) ?? []
        return URL(string: service.nodeUrl(chain: chain.rawValue, selectedUrl: selectedUrl, storedNodes: storedNodes)) ?? chain.defaultBaseUrl
    }
}

// MARK: - Static

public extension NodeService {
    static func isValid(networkId: String, for chain: Chain) -> Bool {
        ChainConfig.config(chain: chain).networkId == networkId
    }
}
