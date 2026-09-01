// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemKeystorePassword
import typealias Gemstone.WalletId

public final class GemstoneKeystorePassword: GemKeystorePassword, @unchecked Sendable {
    private let keystore: any Keystore

    public init(keystore: any Keystore) {
        self.keystore = keystore
    }

    public func getPassword(createIfMissing: Bool) throws -> String {
        try keystore.keystorePassword(createIfMissing: createIfMissing)
    }

    public func getWalletPassword(walletId _: Gemstone.WalletId) throws -> String? {
        nil
    }

    public func deleteWalletPassword(walletId _: Gemstone.WalletId) throws {}
}
