// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Primitives.Chain {
    func checksumAddress(_ address: String) -> String {
        Gemstone.checksumAddress(address: address, chain: rawValue)
    }
}
