// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemPaymentTransaction
import struct Gemstone.PaymentRequest
import GemstonePrimitives
import Primitives

public extension GemPaymentTransaction {
    static func mock(
        memo: String? = nil,
        request: Gemstone.PaymentRequest? = nil,
    ) -> GemPaymentTransaction {
        GemPaymentTransaction(
            merchant: Primitives.ApplicationMetadata(
                name: "Merchant",
                description: "Payment",
                url: "https://example.com",
                icon: "https://example.com/icon.png",
                source: .payment,
            ).toGem(),
            account: Primitives.ChainAddress(chain: .solana, address: "account").toGem(),
            transaction: "encoded-transaction",
            transactionType: Primitives.TransactionType.transfer.toGem(),
            memo: memo,
            request: request,
        )
    }
}
