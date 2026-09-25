// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransactionListItem {
    static func mock(
        transaction: Transaction = .mock(),
        asset: Asset = .mock(),
        assets: [Asset] = [],
        fromAddress: AddressName? = nil,
        toAddress: AddressName? = nil,
    ) -> TransactionListItem {
        TransactionListItem(
            transaction: transaction,
            asset: asset,
            assets: assets,
            fromAddress: fromAddress,
            toAddress: toAddress,
        )
    }
}
