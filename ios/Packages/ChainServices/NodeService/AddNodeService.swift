// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config
import Primitives
import Store

public struct AddNodeService {
    public let nodeStore: NodeStore

    public init(nodeStore: NodeStore) {
        self.nodeStore = nodeStore
    }

    public func addNodes() throws {
        let configNodes = Config.shared.getNodes().map { key, values in
            ChainNodes(chain: key, nodes: values.map(\.node))
        }
        // let existingNodes = try nodeStore.nodes()
        // TODO: Remove outdated nodes (that does not exist in config nodes, except custom added nodes)

        try nodeStore.addNodes(chainNodes: configNodes)
    }

    public func addNode(_ node: ChainNodes) throws {
        try nodeStore.addNodes(chainNodes: [node])
    }
}
