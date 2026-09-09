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
        updatedAt: Date = .now,
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
            updatedAt: updatedAt,
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
        id: String = "mock_id",
        asset: Asset = .mock(),
        transactionType: FiatQuoteType = .buy,
        provider: FiatProviderName = .moonPay,
        status: FiatTransactionStatus = .complete,
        fiatAmount: Double = 100.0,
        fiatCurrency: String = "USD",
        value: String = "0",
        createdAt: Date = .now,
        detailsUrl: String? = nil,
    ) -> FiatTransactionAssetData {
        FiatTransactionAssetData(
            id: id,
            asset: asset,
            transactionType: transactionType,
            provider: provider,
            status: status,
            fiatAmount: fiatAmount,
            fiatCurrency: fiatCurrency,
            value: value,
            createdAt: createdAt,
            detailsUrl: detailsUrl,
        )
    }
}
