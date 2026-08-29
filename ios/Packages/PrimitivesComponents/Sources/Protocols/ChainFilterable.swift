// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemChainServiceProtocol
import Primitives

public protocol ChainFilterable {
    var chainService: any GemChainServiceProtocol { get }
}

public extension ChainFilterable {
    func filterChains(for query: String) -> [Chain] {
        chainService.getChains(query: query).compactMap { Chain(rawValue: $0) }
    }
}
