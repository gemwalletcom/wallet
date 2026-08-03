// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension PaymentRequest {
    static func mock(
        address: String = .empty,
        amount: String? = .none,
        memo: String? = .none,
        assetId: AssetId? = .none,
    ) -> Self {
        .init(
            address: address,
            amount: amount,
            memo: memo,
            assetId: assetId,
        )
    }
}

public extension PaymentLink {
    static func mock(
        provider: PaymentProviderName = .walletConnectPay,
        id: String = "pay_1",
    ) -> Self {
        .init(provider: provider, id: id)
    }
}

public extension PaymentMerchant {
    static func mock(
        name: String = "Test Merchant",
        iconUrl: String? = .none,
    ) -> Self {
        .init(name: name, iconUrl: iconUrl)
    }
}

public extension PaymentOutcome {
    static func mock(
        status: PaymentStatus = .succeeded,
        transactionId: String? = .none,
    ) -> Self {
        .init(status: status, transactionId: transactionId)
    }
}

public extension PaymentQuotes {
    static func mock(
        merchant: PaymentMerchant = .mock(),
        price: PaymentPrice = .mock(),
        expiresAt: Date = Date(timeIntervalSinceNow: 900),
        quotes: [PaymentQuote] = [.mock()],
    ) -> Self {
        .init(merchant: merchant, price: price, expiresAt: expiresAt, quotes: quotes)
    }
}

public extension PaymentQuote {
    static func mock(
        paymentId: String = "pay_1",
        amount: PaymentAmount = .mock(),
        expiresAt: Date? = .none,
        collectDataUrl: String? = .none,
        providerData: String = "{}",
        id: String = "option_1",
    ) -> Self {
        .init(id: id, paymentId: paymentId, amount: amount, expiresAt: expiresAt, collectDataUrl: collectDataUrl, providerData: providerData)
    }
}

public extension PaymentAmount {
    static func mock(
        assetId: AssetId = .mock(),
        value: String = "10000",
        symbol: String = "USDC",
        decimals: Int32 = 6,
    ) -> Self {
        .init(assetId: assetId, value: value, symbol: symbol, decimals: decimals)
    }
}

public extension PaymentPrice {
    static func mock(
        symbol: String = "USD",
        value: String = "1",
        decimals: Int32 = 2,
    ) -> PaymentPrice {
        PaymentPrice(symbol: symbol, value: value, decimals: decimals)
    }
}
