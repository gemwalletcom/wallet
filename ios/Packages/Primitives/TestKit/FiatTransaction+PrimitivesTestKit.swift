// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension FiatTransaction {
    static func mock(
        id: String = "mock_id",
        assetId: AssetId = .mock(),
        transactionType: FiatQuoteType = .buy,
        provider: FiatProviderName = .moonPay,
        status: FiatTransactionStatus = .complete,
        fiatAmount: Double = 100.0,
        fiatCurrency: String = "USD",
        value: String = "0",
        createdAt: Date = .now,
    ) -> FiatTransaction {
        FiatTransaction(
            id: id,
            assetId: assetId,
            transactionType: transactionType,
            provider: provider,
            status: status,
            fiatAmount: fiatAmount,
            fiatCurrency: fiatCurrency,
            value: value,
            createdAt: createdAt,
        )
    }
}

public extension FiatTransactionData {
    static func mock(
        transaction: FiatTransaction = .mock(),
        detailsUrl: String? = nil,
    ) -> FiatTransactionData {
        FiatTransactionData(
            transaction: transaction,
            detailsUrl: detailsUrl,
        )
    }
}

public extension FiatTransactionAssetData {
    static func mock(
        transaction: FiatTransaction = .mock(),
        asset: Asset = .mock(),
        detailsUrl: String? = nil,
    ) -> FiatTransactionAssetData {
        FiatTransactionAssetData(
            transaction: transaction,
            asset: asset,
            detailsUrl: detailsUrl,
        )
    }
}
