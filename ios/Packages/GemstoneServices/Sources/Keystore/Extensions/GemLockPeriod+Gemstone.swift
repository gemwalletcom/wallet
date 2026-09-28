// Copyright (c). Gem Wallet. All rights reserved.

public import enum Gemstone.GemLockPeriod
import func Gemstone.lockPeriodFromMinutes

extension GemLockPeriod: @retroactive Identifiable {
    public var id: Self {
        self
    }
}

public extension GemLockPeriod {
    static var `default`: GemLockPeriod {
        lockPeriodFromMinutes(minutes: nil)
    }
}
