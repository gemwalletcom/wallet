// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives

extension NameRecord {
    func isValidRecipient(name: String, chain: Chain) -> Bool {
        self.name == name &&
            self.chain == chain &&
            chain.isValidAddress(address)
    }
}
