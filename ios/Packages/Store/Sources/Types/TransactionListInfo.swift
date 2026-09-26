// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct TransactionListInfo: Codable, FetchableRecord {
    let transaction: TransactionRecord
    let asset: AssetRecord
    let assets: [AssetRecord]
    let fromAddress: AddressRecord?
    let toAddress: AddressRecord?
}

extension TransactionListInfo {
    func mapToTransactionListItem() -> TransactionListItem {
        TransactionListItem(
            transaction: transaction.mapToTransaction(),
            asset: asset.mapToAsset(),
            assets: assets.map { $0.mapToAsset() },
            fromAddress: fromAddress?.mapToAddressName(),
            toAddress: toAddress?.mapToAddressName(),
        )
    }
}
