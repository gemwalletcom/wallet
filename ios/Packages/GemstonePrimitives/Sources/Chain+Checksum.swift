// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Primitives.Chain {
    func checksumAddress(_ address: String) throws -> String {
        try GemChainAddress(address: address, chain: rawValue).address()
    }
}
