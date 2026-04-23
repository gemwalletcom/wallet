// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.Transaction {
    func map() -> Gemstone.TransactionStateInput {
        Gemstone.TransactionStateInput(
            chain: assetId.chain.rawValue,
            hash: id.hash,
            from: from,
            createdAtSecs: Int64(createdAt.timeIntervalSince1970),
            blockNumber: Int64(blockNumber ?? "0") ?? 0,
            swapMetadata: nil,
        )
    }
}

public extension Primitives.TransactionWallet {
    func map() -> Gemstone.TransactionStateInput {
        transaction.map()
    }
}
