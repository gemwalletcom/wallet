// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Chain {
    var defaultBaseUrl: URL {
        Constants.nodesURL.appending(component: rawValue.lowercased())
    }

    var defaultNode: Node {
        Node(url: defaultBaseUrl.absoluteString, status: .active, priority: 10)
    }

    var defaultChainNode: ChainNode {
        ChainNode(chain: rawValue, node: defaultNode)
    }

    var europeChainNode: ChainNode {
        ChainNode(
            chain: rawValue,
            node: Node(
                url: Constants.nodesEuropeURL.appending(component: rawValue.lowercased()).absoluteString,
                status: .active,
                priority: 9,
            ),
        )
    }

    var asiaChainNode: ChainNode {
        ChainNode(
            chain: rawValue,
            node: Node(
                url: Constants.nodesAsiaURL.appending(component: rawValue.lowercased()).absoluteString,
                status: .active,
                priority: 8,
            ),
        )
    }
}
