// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Chain
import protocol Gemstone.GemNodeStore
import struct Gemstone.Node
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneNodeStore: GemNodeStore, Sendable {
    private let store: NodeStore

    public init(store: NodeStore) {
        self.store = store
    }

    public func getNodes(chain: Gemstone.Chain) async throws -> [Gemstone.Node] {
        try store.nodes(chain: Primitives.Chain(id: chain)).map { $0.node.toGem() }
    }

    public func addNode(chain: Gemstone.Chain, node: Gemstone.Node) async throws {
        try store.addNodes(chainNodes: [ChainNodes(chain: chain, nodes: [node.toPrimitives()])])
    }

    public func deleteNode(chain: Gemstone.Chain, url: String) async throws {
        try store.deleteNode(chain: Primitives.Chain(id: chain), url: url)
    }
}
