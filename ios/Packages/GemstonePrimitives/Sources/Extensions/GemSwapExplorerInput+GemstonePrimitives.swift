// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSwapExplorerInput
import Primitives

public extension Primitives.SwapExplorerInput {
    func map() -> Gemstone.GemSwapExplorerInput {
        Gemstone.GemSwapExplorerInput(txHash: tx_hash, recipient: recipient, memo: memo)
    }
}
