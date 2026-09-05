// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension FiatQuote {
    static func mock(
        id: String = UUID().uuidString,
        fiatAmount: Double = 0,
        cryptoAmount: Double = 0,
        type: FiatQuoteType = .buy,
        fiatCurrency: String = Currency.usd.rawValue,
        providerId: String = "moonpay",
    ) -> FiatQuote {
        FiatQuote(
            id: id,
            asset: .mock(),
            provider: FiatProvider(id: providerId, name: "", imageUrl: "", paymentMethods: []),
            type: type,
            fiatAmount: fiatAmount,
            fiatCurrency: fiatCurrency,
            cryptoAmount: cryptoAmount,
            paymentMethods: [],
        )
    }
}
