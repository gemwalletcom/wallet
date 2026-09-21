// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct TransactionAssets: Sendable {
    public let transaction: Transaction
    public let assetIds: [AssetId]

    public init(transaction: Transaction, assetIds: [AssetId]) {
        self.transaction = transaction
        self.assetIds = assetIds
    }
}
