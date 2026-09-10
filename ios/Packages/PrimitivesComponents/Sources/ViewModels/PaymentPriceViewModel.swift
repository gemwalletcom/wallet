// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import struct Gemstone.PaymentPrice

public struct PaymentPriceViewModel {
    private let price: PaymentPrice

    public init(price: PaymentPrice) {
        self.price = price
    }

    public var text: String {
        CurrencyFormatter(currencyCode: price.currency).string(price.amount)
    }
}
