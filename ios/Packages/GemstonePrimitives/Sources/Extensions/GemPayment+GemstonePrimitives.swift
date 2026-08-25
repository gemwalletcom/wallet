// Copyright (c). Gem Wallet. All rights reserved.

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
            references: references,
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
        case let .solanaPay(url): .solanaPay(PaymentLinkSolanaPayInner(url: url))
        }
    }
}

public extension PaymentLink {
    func map() -> GemPaymentLink {
        switch self {
        case let .solanaPay(link): .solanaPay(url: link.url)
        }
    }
}

public extension PaymentRequest {
    func map() -> GemPaymentRequest {
        GemPaymentRequest(
            address: address,
            amount: amount?.map(),
            memo: memo,
            references: references,
            assetId: assetId?.identifier,
        )
    }
}

public extension PaymentAmount {
    func map() -> GemPaymentAmount {
        switch self {
        case let .exactValue(value): .exactValue(value)
        case let .atomicValue(value): .atomicValue(value)
        }
    }
}
