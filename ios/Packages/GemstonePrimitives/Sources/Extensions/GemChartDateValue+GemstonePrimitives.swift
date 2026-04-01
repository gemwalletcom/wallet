// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemChartDateValue {
    func map() -> ChartDateValue {
        ChartDateValue(
            date: Date(timeIntervalSince1970: TimeInterval(date)),
            value: value,
        )
    }
}
