// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct ChainNode: Codable, Sendable {
    public let chain: String
    public let node: Node

    public init(chain: String, node: Node) {
        self.chain = chain
        self.node = node
    }
}

public struct ChainNodes: Codable, Sendable {
    public let chain: String
    public let nodes: [Node]

    public init(chain: String, nodes: [Node]) {
        self.chain = chain
        self.nodes = nodes
    }
}
