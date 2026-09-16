// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
@testable import Perpetuals
import Primitives
import PrimitivesTestKit

public extension CandleTooltipViewModel {
    static func mock(candle: ChartCandleStick = .mock()) -> CandleTooltipViewModel {
        CandleTooltipViewModel(candle: candle)
    }
}
