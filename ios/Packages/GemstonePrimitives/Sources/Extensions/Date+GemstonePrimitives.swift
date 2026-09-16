// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDay
import struct Gemstone.GemDayBoundaries

public extension Date {
    var gemDay: GemDay {
        let components = Calendar.current.dateComponents([.year, .month, .day], from: self)
        return GemDay(
            year: Int32(components.year ?? 0),
            month: UInt32(components.month ?? 0),
            day: UInt32(components.day ?? 0),
        )
    }
}

public extension GemDayBoundaries {
    static var current: GemDayBoundaries {
        Date.now.gemDay.boundaries()
    }
}
