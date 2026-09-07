// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct TransactionInfo: Codable, FetchableRecord {
    let transaction: TransactionRecord
    let asset: AssetRecord
    let feeAsset: AssetRecord
    let price: PriceRecord?
    let feePrice: PriceRecord?
    let assets: [AssetRecord]
    let prices: [PriceRecord]
    let fromAddress: AddressRecord?
    let toAddress: AddressRecord?
}

extension TransactionInfo {
    func mapToTransactionExtended() throws -> TransactionExtended {
        guard let recordId = transaction.id else {
            throw RecordError.recordNotFound(databaseTableName: TransactionRecord.databaseTableName, key: [:])
        }
        return TransactionExtended(
            recordId: UInt64(recordId),
            transaction: transaction.mapToTransaction(),
            asset: asset.mapToAsset(),
            feeAsset: feeAsset.mapToAsset(),
            price: price?.mapToPrice(),
            feePrice: feePrice?.mapToPrice(),
            assets: assets.map { $0.mapToAsset() },
            prices: prices.compactMap { $0.mapToAssetPrice() },
            fromAddress: fromAddress?.mapToAddressName(),
            toAddress: toAddress?.mapToAddressName(),
            confirmationEtaSeconds: transaction.confirmationEtaSeconds,
        )
    }
}
