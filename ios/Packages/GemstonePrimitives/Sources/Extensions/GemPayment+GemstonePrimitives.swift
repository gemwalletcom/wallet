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
            amount: amount,
            memo: memo,
            assetId: assetId.map { try AssetId(id: $0) },
        )
    }
}

public extension GemPaymentLink {
    func map() -> PaymentLink {
        switch self {
        case let .solanaPay(id): .solanaPay(id)
        }
    }
}
