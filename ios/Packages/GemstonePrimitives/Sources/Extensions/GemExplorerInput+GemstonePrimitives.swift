// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemExplorerInput
import Primitives

public extension Primitives.ExplorerInput {
    func map() -> Gemstone.GemExplorerInput {
        Gemstone.GemExplorerInput(hash: hash, recipient: recipient, memo: memo)
    }
}
