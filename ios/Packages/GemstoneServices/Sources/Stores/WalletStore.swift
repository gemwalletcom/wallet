// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Wallet
import typealias Gemstone.WalletId
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

    public func getWallet(walletId: Gemstone.WalletId) async throws -> Gemstone.Wallet? {
        try store.getWallet(id: Primitives.WalletId.from(id: walletId)).map { try $0.json() }
    }

    public func nextWalletIndex() async throws -> Int32 {
        try Int32(store.nextWalletIndex())
    }

    public func addWallet(wallet: Gemstone.Wallet) async throws {
        try store.addWallet(Primitives.Wallet(wallet))
    }

    public func deleteWallet(walletId: Gemstone.WalletId) async throws -> Bool {
        try store.deleteWallet(for: Primitives.WalletId.from(id: walletId))
    }

    public func setPinned(walletId: Gemstone.WalletId, pinned: Bool) async throws {
        try store.pinWallet(Primitives.WalletId.from(id: walletId), value: pinned)
    }

    public func rename(walletId: Gemstone.WalletId, name: String) async throws {
        try store.renameWallet(Primitives.WalletId.from(id: walletId), name: name)
    }
}
