// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPaymentOptions {
    func map() throws -> PaymentOptions {
        switch self {
        case let .quotes(quotes): try .quotes(quotes.map())
        case let .outcome(outcome): .outcome(outcome.map())
        }
    }
}

public extension PaymentQuotes {
    func map() -> GemPaymentQuotes {
        GemPaymentQuotes(
            merchant: merchant.map(),
            price: price?.map(),
            expiresAt: expiresAt.map { Int64($0.timeIntervalSince1970) },
            quotes: quotes.map { $0.map() },
        )
    }
}

public extension PaymentMerchant {
    func map() -> GemPaymentMerchant {
        GemPaymentMerchant(name: name, iconUrl: iconUrl)
    }
}

public extension PaymentPrice {
    func map() -> GemPaymentPrice {
        GemPaymentPrice(symbol: symbol, value: value, decimals: decimals)
    }
}

public extension GemPaymentQuotes {
    func map() throws -> PaymentQuotes {
        try PaymentQuotes(
            merchant: merchant.map(),
            price: price?.map(),
            expiresAt: expiresAt.map { Date(timeIntervalSince1970: TimeInterval($0)) },
            quotes: quotes.map { try $0.map() },
        )
    }
}

public extension GemPaymentQuote {
    func map() throws -> PaymentQuote {
        try PaymentQuote(
            id: id,
            paymentId: paymentId,
            amount: amount.map(),
            expiresAt: expiresAt.map { Date(timeIntervalSince1970: TimeInterval($0)) },
            collectDataUrl: collectDataUrl,
            providerData: providerData,
        )
    }
}

public extension PaymentQuote {
    func map() -> GemPaymentQuote {
        GemPaymentQuote(
            id: id,
            paymentId: paymentId,
            amount: amount.map(),
            expiresAt: expiresAt.map { Int64($0.timeIntervalSince1970) },
            collectDataUrl: collectDataUrl,
            providerData: providerData,
        )
    }
}

public extension PaymentAmount {
    func map() -> GemPaymentAmount {
        GemPaymentAmount(assetId: assetId.identifier, value: value, symbol: symbol, decimals: decimals)
    }
}

public extension GemPaymentAmount {
    func map() throws -> PaymentAmount {
        try PaymentAmount(assetId: AssetId(id: assetId), value: value, symbol: symbol, decimals: decimals)
    }
}

public extension GemPaymentMerchant {
    func map() -> PaymentMerchant {
        PaymentMerchant(name: name, iconUrl: iconUrl)
    }
}

public extension GemPaymentOutcome {
    func map() -> PaymentOutcome {
        PaymentOutcome(status: status.map(), transactionId: transactionId)
    }
}

public extension GemPaymentStatus {
    func map() -> PaymentStatus {
        switch self {
        case .requiresAction: .requiresAction
        case .processing: .processing
        case .succeeded: .succeeded
        case .failed: .failed
        case .expired: .expired
        case .cancelled: .cancelled
        }
    }
}

public extension GemPaymentPrice {
    func map() -> PaymentPrice {
        PaymentPrice(symbol: symbol, value: value, decimals: decimals)
    }
}
