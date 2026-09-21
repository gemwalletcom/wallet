// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Transaction {
    static func id(chain: Chain, hash: String) -> String {
        String(format: "%@_%@", chain.rawValue, hash)
    }

    var chain: Chain {
        assetId.chain
    }
}

extension Transaction: Identifiable {}
