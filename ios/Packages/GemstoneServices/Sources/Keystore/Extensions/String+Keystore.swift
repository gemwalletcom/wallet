// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

extension String {
    func v4KeystorePasswordBytes() throws -> Data {
        // Wallets created before the hex generator used UUID().uuidString passwords, which aren't valid hex.
        // WalletCore consumed the password as raw utf8 bytes, so fall back to utf8 for those legacy passwords;
        // otherwise they fail v3->v4 migration and signing with "invalid hex value".
        (try? Data.from(hex: self)) ?? Data(utf8)
    }

    func v3PasswordBytes() -> Data {
        Data(utf8)
    }
}
