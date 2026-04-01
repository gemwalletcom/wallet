// Copyright (c). Gem Wallet. All rights reserved.

import ChainServiceTestKit
import Foundation
import protocol Gemstone.GemSwapperProtocol
import SwapService

public extension SwapService {
    static func mock(swapper: GemSwapperProtocol = GemSwapperMock()) -> SwapService {
        SwapService(swapper: swapper)
    }
}
