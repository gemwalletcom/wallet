// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SwapperSlippage
import enum Gemstone.SwapperSlippageMode
import Primitives

public extension SwapperSlippage {
    init(slippage: SwapSlippage, defaultBps: UInt32) {
        switch slippage {
        case .auto:
            self.init(bps: defaultBps, mode: .auto)
        case let .manual(bps):
            self.init(bps: bps, mode: .exact)
        }
    }
}
