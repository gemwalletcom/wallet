// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension BitcoinChain {
    init(id: String) throws {
        if let chain = BitcoinChain(rawValue: id) {
            self = chain
        } else {
            throw AnyError("invalid chain id: \(id)")
        }
    }

    var chain: Chain {
        Chain(rawValue: rawValue)!
    }
}
