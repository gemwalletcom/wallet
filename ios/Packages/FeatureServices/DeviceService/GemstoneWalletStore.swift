// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Wallet
import protocol Gemstone.GemWalletStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneWalletStore: GemWalletStore, @unchecked Sendable {
    private let store: WalletStore

    public init(store: WalletStore) {
        self.store = store
    }

    public func getWallets() async throws -> [Gemstone.Wallet] {
        try store.getWallets().map { try $0.json() }
    }

    public func getWallet(walletId: String) async throws -> Gemstone.Wallet? {
        try store.getWallet(id: WalletId.from(id: walletId)).map { try $0.json() }
    }
}
