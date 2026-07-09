// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SwapperSlippage
import enum Gemstone.SwapperSlippageMode
import Primitives

public extension SwapperSlippage {
    init(slippage: SwapSlippage, defaultSlippage: SwapperSlippage) {
        switch slippage {
        case .auto:
            self.init(bps: defaultSlippage.bps, mode: .auto)
        case let .manual(bps):
            self.init(bps: bps, mode: .exact)
        }
    }
}
