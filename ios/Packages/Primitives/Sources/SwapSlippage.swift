// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum SwapSlippage: Hashable, Sendable {
    case auto
    case manual(bps: UInt32)

    public var isCustom: Bool {
        switch self {
        case .auto: false
        case .manual: true
        }
    }
}
