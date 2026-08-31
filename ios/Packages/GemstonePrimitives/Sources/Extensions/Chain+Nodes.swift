// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.NodeRegion
import Primitives

public extension Chain {
    var defaultBaseUrl: URL {
        NodeURL.url(chain: self, region: .us)
    }

    var defaultChainNode: ChainNode {
        chainNode(region: .us)
    }

    private func node(region: NodeRegion) -> Node {
        Node(
            url: NodeURL.url(chain: self, region: region).absoluteString,
            status: .active,
            priority: NodeURL.priority(region: region),
        )
    }

    func chainNode(region: NodeRegion) -> ChainNode {
        ChainNode(chain: rawValue, node: node(region: region))
    }
}
