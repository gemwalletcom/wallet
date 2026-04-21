// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.Account {
    var gemAccount: GemAccount {
        GemAccount(
            chain: chain.id,
            address: address,
            derivationPath: derivationPath,
            publicKey: publicKey,
            extendedPublicKey: extendedPublicKey,
        )
    }
}
