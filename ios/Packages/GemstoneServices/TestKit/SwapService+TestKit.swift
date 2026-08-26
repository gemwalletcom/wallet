// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSwapperProtocol
import GemstoneServices

public extension SwapService {
    static func mock(swapper: GemSwapperProtocol = GemSwapperMock()) -> SwapService {
        SwapService(swapper: swapper)
    }
}
