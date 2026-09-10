// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.ChainAddress
import Primitives

public extension Primitives.Wallet {
    var chainAddresses: [Gemstone.ChainAddress] {
        accounts.map { Primitives.ChainAddress(chain: $0.chain, address: $0.address).map() }
    }
}
