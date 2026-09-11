// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import enum Gemstone.GemAmountSign
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension AmountDisplay {
    static func mock(
        asset: Asset = Asset.mock(),
        price: Price? = Price.mock(price: 1.0),
        value: BigInt = BigInt(100_000_000),
        sign: GemAmountSign = .none,
        currency: String = "USD",
        formatter: ValueFormatter = .full,
    ) -> AmountDisplay {
        .numeric(
            asset: asset,
            price: price,
            value: value,
            sign: sign,
            currency: currency,
            formatter: formatter,
        )
    }
}
