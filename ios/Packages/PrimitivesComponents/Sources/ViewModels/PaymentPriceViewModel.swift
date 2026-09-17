// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.PaymentPrice
import GemstonePrimitives

public struct PaymentPriceViewModel {
    private let price: PaymentPrice

    public init(price: PaymentPrice) {
        self.price = price
    }

    public var text: String {
        CurrencyFormatter(currencyCode: price.currency).string(price.amount)
    }
}
