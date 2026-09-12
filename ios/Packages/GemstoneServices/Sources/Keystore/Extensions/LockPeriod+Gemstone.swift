// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemLockPeriod
import func Gemstone.lockPeriods
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

public extension GemLockPeriod {
    var lockPeriod: LockPeriod {
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

public extension LockPeriod {
    static var offered: [LockPeriod] {
        lockPeriods().map { $0.lockPeriod }
    }
}
