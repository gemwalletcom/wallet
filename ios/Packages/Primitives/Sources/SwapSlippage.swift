// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum SwapSlippage: Hashable, Sendable {
    case auto
    case manual(bps: UInt32)
}
