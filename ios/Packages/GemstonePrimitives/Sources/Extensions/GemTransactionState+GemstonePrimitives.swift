// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Gemstone.TransactionState {
    func map() -> Primitives.TransactionState {
        switch self {
        case .pending: .pending
        case .confirmed: .confirmed
        case .inTransit: .inTransit
        case .failed: .failed
        case .reverted: .reverted
        }
    }
}
