// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import class Gemstone.GemAutocloseEstimator
import GemstonePrimitives
import Perpetuals
import Primitives
import PrimitivesComponents

public extension AutocloseViewModel {
    static func mock(
        type: TpslType = .takeProfit,
        price: Double? = nil,
        positionSize: Double = 10.0,
        leverage: UInt8 = 5,
    ) -> AutocloseViewModel {
        AutocloseViewModel(
            type: type,
            price: price,
            estimator: GemAutocloseEstimator(
                entryPrice: 100.0,
                positionSize: positionSize,
                direction: Primitives.PerpetualDirection.long.toGem(),
                leverage: leverage,
            ),
        )
    }
}
