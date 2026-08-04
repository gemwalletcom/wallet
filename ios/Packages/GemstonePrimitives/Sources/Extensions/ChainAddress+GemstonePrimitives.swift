// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.ChainAddress {
    func map() -> Gemstone.ChainAddress {
        Gemstone.ChainAddress(chain: chain.rawValue, address: address)
    }
}
