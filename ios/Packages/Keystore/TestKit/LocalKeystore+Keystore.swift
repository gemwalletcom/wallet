// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore

/// For public use
public extension LocalKeystore {
    static let words = ["shoot", "island", "position", "soft", "burden", "budget", "tooth", "cruel", "issue", "economy", "destroy", "above"]
    static let privateKey = "0x9f110a73d04dc7becb316fb9adfe04689a947bb49be11060577c3c0a4b4d4cd5"
    static let address = "0x734dC149D4c7D0D5E95B5AA787e5FB288dD167a9"
    static let bitcoinAddress = "bc1quvuarfksewfeuevuc6tn0kfyptgjvwsvrprk9d"
    static let password = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

    static func mock(
        keystorePassword: KeystorePassword = MockKeystorePassword(),
    ) -> LocalKeystore {
        LocalKeystore(
            directory: UUID().uuidString,
            keystorePassword: keystorePassword,
        )
    }
}
