// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemNodeService
import GemstoneStore
import NodeService
import Store
import StoreTestKit

public extension NodeService {
    static func mock(
        nodeStore: NodeStore = .mock(),
    ) -> NodeService {
        NodeService(
            nodeStore: nodeStore,
            service: GemNodeService(store: GemstoneNodeStore(store: nodeStore)),
        )
    }
}
