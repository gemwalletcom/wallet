// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletDeletion
import protocol Gemstone.GemWalletServiceProtocol
import Primitives

public extension GemWalletServiceProtocol {
    var currentWalletId: WalletId? {
        do {
            return try currentWalletId().map { try WalletId.from(id: $0) }
        } catch {
            debugLog("current wallet id unavailable: \(error)")
            return .none
        }
    }

    func sorted(wallets: [Wallet]) -> [Wallet] {
        sortedWallets(wallets: wallets.map { $0.toGem() }).map { $0.toPrimitives() }
    }

    func delete(_ wallet: Wallet) async throws -> GemWalletDeletion {
        try await deleteWallet(walletId: wallet.id.id)
    }

    func pin(wallet: Wallet) async throws {
        try await setPinned(walletId: wallet.id.id, pinned: true)
    }

    func unpin(wallet: Wallet) async throws {
        try await setPinned(walletId: wallet.id.id, pinned: false)
    }

    func rename(walletId: WalletId, newName: String) async throws {
        try await rename(walletId: walletId.id, name: newName)
    }

    func setImage(data: Data, for wallet: Wallet) async throws {
        try await setAvatarImage(walletId: wallet.id.id, image: data)
    }

    func setImage(url: URL, for wallet: Wallet) async throws {
        try await setAvatarImageUrl(walletId: wallet.id.id, url: url.absoluteString)
    }

    func removeImage(for wallet: Wallet) async throws {
        try await removeAvatarImage(walletId: wallet.id.id)
    }
}
