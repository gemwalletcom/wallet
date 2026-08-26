// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemKeystorePassword
import typealias Gemstone.WalletId
import Keystore

public final class GemstoneKeystorePassword: GemKeystorePassword, @unchecked Sendable {
    private let keystore: any Keystore

    public init(keystore: any Keystore) {
        self.keystore = keystore
    }

    public func getPassword(walletId _: Gemstone.WalletId, createIfMissing: Bool) throws -> Data {
        try keystore.keystorePassword(createIfMissing: createIfMissing)
    }
}
