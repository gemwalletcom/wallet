// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSlippageSelection

public extension GemSlippageSelection {
    var bps: UInt32? {
        switch self {
        case .auto: nil
        case let .manual(bps): bps
        }
    }

    var isCustom: Bool {
        switch self {
        case .auto: false
        case .manual: true
        }
    }
}
