// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Perpetuals
import Primitives
import class Gemstone.GemAutocloseEstimator
import GemstonePrimitives

public extension AutocloseViewModel {
    static func mock(
        type: TpslType = .takeProfit,
        price: Double? = nil,
        positionSize: Double = 10.0,
        leverage: UInt8 = 5,
        currencyFormatter: CurrencyFormatter = CurrencyFormatter(currencyCode: "USD"),
        percentFormatter: PercentFormatter = .signed,
    ) -> AutocloseViewModel {
        AutocloseViewModel(
            type: type,
            price: price,
            estimator: GemAutocloseEstimator(
                entryPrice: 100.0,
                positionSize: positionSize,
                direction: Primitives.PerpetualDirection.long.map(),
                leverage: leverage,
            ),
            currencyFormatter: currencyFormatter,
            percentFormatter: percentFormatter,
        )
    }
}
