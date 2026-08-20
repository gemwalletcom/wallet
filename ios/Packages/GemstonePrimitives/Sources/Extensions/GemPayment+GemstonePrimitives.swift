// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPayment {
    func map() throws -> Payment {
        switch self {
        case let .request(request): try .request(request.map())
        case let .link(link): .link(link.map())
        }
    }
}

public extension GemPaymentRequest {
    func map() throws -> PaymentRequest {
        try PaymentRequest(
            address: address,
            amount: amount?.map(),
            memo: memo,
            assetId: assetId.map { try AssetId(id: $0) },
        )
    }
}

public extension GemPaymentAmount {
    func map() -> PaymentAmount {
        switch self {
        case let .exactValue(value): .exactValue(value)
        case let .atomicValue(value): .atomicValue(value)
        }
    }
}

public extension GemPaymentLink {
    func map() -> PaymentLink {
        switch self {
        case let .solanaPay(id): .solanaPay(id)
        case let .walletConnectPay(id): .walletConnectPay(id)
        }
    }
}

public extension PaymentLink {
    func map() -> GemPaymentLink {
        switch self {
        case let .solanaPay(id): .solanaPay(id)
        case let .walletConnectPay(id): .walletConnectPay(id)
        }
    }
}

public extension GemPaymentOptions {
    func map() throws -> PaymentOptions {
        switch self {
        case let .quotes(quotes): try .quotes(quotes.map())
        case let .outcome(outcome): .outcome(outcome.map())
        }
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
            link: link.map(),
            assetId: AssetId(id: assetId),
            value: value,
            expiresAt: expiresAt.map { Date(timeIntervalSince1970: TimeInterval($0)) },
            collectDataUrl: collectDataUrl,
            providerData: providerData,
        )
    }
}

public extension Primitives.PaymentQuote {
    func map() -> GemPaymentQuote {
        GemPaymentQuote(
            id: id,
            link: link.map(),
            assetId: assetId.identifier,
            value: value,
            expiresAt: expiresAt.map { Int64($0.timeIntervalSince1970) },
            collectDataUrl: collectDataUrl,
            providerData: providerData,
        )
    }
}

public extension GemPaymentPrice {
    func map() -> PaymentPrice {
        PaymentPrice(symbol: symbol, value: value, decimals: decimals)
    }
}

public extension GemPaymentMerchant {
    func map() -> PaymentMerchant {
        PaymentMerchant(name: name, iconUrl: iconUrl)
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

public extension GemPaymentOutcome {
    func map() -> PaymentOutcome {
        PaymentOutcome(status: status.map(), transactionId: transactionId)
    }
}

public extension GemPaymentAction {
    func map() throws -> PaymentAction {
        switch self {
        case let .send(chain, recipient, value, data):
            guard let chain = Primitives.Chain(rawValue: chain) else {
                throw AnyError("Unsupported payment chain: \(chain)")
            }
            return .send(PaymentActionSendInner(chain: chain, recipient: recipient, value: value, data: data))
        }
    }
}

public extension GemPaymentQuoteData {
    func map() throws -> PaymentQuoteData {
        try PaymentQuoteData(quote: quote.map(), action: action.map())
    }
}

public extension PaymentMerchant {
    var appMetadata: WalletConnectionSessionAppMetadata {
        WalletConnectionSessionAppMetadata(
            name: name,
            description: .empty,
            url: Gemstone.paymentWalletConnectUrl(),
            icon: iconUrl ?? .empty,
        )
    }
}
