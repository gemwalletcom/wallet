// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Transaction
import func Gemstone.transactionAssetIds
import GemstonePrimitives
import Primitives
import Store

extension Gemstone.Transaction {
    var transactionAssets: TransactionAssets {
        TransactionAssets(
            transaction: toPrimitives(),
            assetIds: transactionAssetIds(transaction: self).map { Primitives.AssetId(core: $0) },
        )
    }
}
