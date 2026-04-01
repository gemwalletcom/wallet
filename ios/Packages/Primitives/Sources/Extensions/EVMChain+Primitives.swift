// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension EVMChain {
    var chain: Chain {
        Chain(rawValue: rawValue)!
    }

    init(from chain: Chain) throws {
        guard let evmChain = EVMChain(rawValue: chain.rawValue) else {
            throw AnyError("Not EVM compatible chain!")
        }
        self = evmChain
    }
}
