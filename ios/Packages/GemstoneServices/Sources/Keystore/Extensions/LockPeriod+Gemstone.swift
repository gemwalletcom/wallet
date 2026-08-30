// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemLockPeriod
import Primitives

public extension LockPeriod {
    var gemLockPeriod: GemLockPeriod {
        switch self {
        case .immediate: .immediate
        case .oneMinute: .oneMinute
        case .fiveMinutes: .fiveMinutes
        case .fifteenMinutes: .fifteenMinutes
        case .oneHour: .oneHour
        case .sixHours: .sixHours
        }
    }
}
