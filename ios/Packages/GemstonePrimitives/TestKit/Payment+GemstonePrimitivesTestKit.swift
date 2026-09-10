// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.PaymentInvoice
import enum Gemstone.PaymentLink
import struct Gemstone.PaymentMerchant
import struct Gemstone.PaymentPrice
import struct Gemstone.PaymentQuote
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension PaymentInvoice {
    static func mock(
        link: PaymentLink = .walletConnectPay(paymentId: "pay_123"),
        merchant: PaymentMerchant = .mock(),
        price: PaymentPrice? = .mock(),
        quotes: [PaymentQuote] = [.mock()],
    ) -> PaymentInvoice {
        PaymentInvoice(link: link, merchant: merchant, price: price, quotes: quotes)
    }
}

public extension PaymentMerchant {
    static func mock(name: String = "Merchant") -> PaymentMerchant {
        PaymentMerchant(name: name, icon: "https://example.com/icon.png")
    }
}

public extension PaymentPrice {
    static func mock(currency: String = "USD", amount: Double = 0.30) -> PaymentPrice {
        PaymentPrice(currency: currency, amount: amount)
    }
}

public extension PaymentQuote {
    static func mock(asset: Asset = .mockEthereum(), value: BigUInt = BigUInt(1_000_000_000_000_000)) -> PaymentQuote {
        PaymentQuote(id: "option-\(asset.id.identifier)", assetId: asset.id.identifier, value: value)
    }
}
