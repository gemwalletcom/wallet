// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletStore
import struct Gemstone.Wallet
import typealias Gemstone.WalletId
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneWalletStore: GemWalletStore, @unchecked Sendable {
    private let store: WalletStore

    public init(store: WalletStore) {
        self.store = store
    }

    public func getWallets() throws -> [Gemstone.Wallet] {
        try store.getWallets().map { $0.toGem() }
    }

    public func getWallet(walletId: Gemstone.WalletId) throws -> Gemstone.Wallet? {
        try store.getWallet(id: walletId).map { $0.toGem() }
    }

    public func addWallet(wallet: Gemstone.Wallet) async throws {
        try store.addWallet(wallet.toPrimitives())
    }

    public func deleteWallet(walletId: Gemstone.WalletId) async throws -> Bool {
        try store.deleteWallet(for: walletId)
    }

    public func setPinned(walletId: Gemstone.WalletId, pinned: Bool) async throws {
        try store.pinWallet(walletId, value: pinned)
    }

    public func setImageUrl(walletId: Gemstone.WalletId, imageUrl: String?) async throws {
        try store.setWalletAvatar(walletId, path: imageUrl)
    }

    public func setName(walletId: Gemstone.WalletId, name: String) async throws {
        try store.renameWallet(walletId, name: name)
    }
}
