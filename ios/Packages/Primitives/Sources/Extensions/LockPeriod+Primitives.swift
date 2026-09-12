// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension LockPeriod {
    public static let `default`: LockPeriod = .oneMinute
}

extension LockPeriod: Identifiable {
    public var id: Self {
        self
    }
}
