// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives

public protocol ChainFilterable {
    var chains: [Chain] { get }
}

public extension ChainFilterable {
    func filterChains(for query: String) -> [Chain] {
        chains.filter(query: query)
    }
}
