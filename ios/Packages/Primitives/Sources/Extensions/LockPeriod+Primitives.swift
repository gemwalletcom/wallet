// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension LockPeriod: CaseIterable {
    public static let allCases: [LockPeriod] = [.immediate, .oneMinute, .fiveMinutes, .fifteenMinutes, .oneHour, .sixHours]
    public static let `default`: LockPeriod = .oneMinute
}

extension LockPeriod: Identifiable {
    public var id: Self {
        self
    }
}
