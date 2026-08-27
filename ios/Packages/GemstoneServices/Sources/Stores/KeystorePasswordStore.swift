// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemKeystorePassword
import typealias Gemstone.WalletId
import Store

public final class GemstoneKeystorePassword: GemKeystorePassword, @unchecked Sendable {
    private let keystore: any Keystore
    private let walletStore: WalletStore

    public init(keystore: any Keystore, walletStore: WalletStore) {
        self.keystore = keystore
        self.walletStore = walletStore
    }

    public func getPassword(walletId _: Gemstone.WalletId, createIfMissing: Bool) throws -> Data {
        try keystore.keystorePassword(createIfMissing: createIfMissing && walletStore.getWallets().allSatisfy { $0.type == .view })
    }
}
