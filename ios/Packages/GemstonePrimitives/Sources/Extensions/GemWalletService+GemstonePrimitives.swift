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

    func nextWalletIndex() throws -> Int {
        Int(try nextWalletIndex() as Int32)
    }

    func sorted(wallets: [Wallet]) -> [Wallet] {
        do {
            return try sortedWallets(wallets: wallets.map { $0.json() }).map { try Wallet($0) }
        } catch {
            preconditionFailure("Undecodable sorted wallets: \(error)")
        }
    }

    func importWallet(name: String, type: KeystoreImportType, source: Primitives.WalletSource) async throws -> WalletImportResult {
        let walletImport = try type.walletImport.validated()
        return switch try await importWallet(name: name, import: walletImport, source: source.map()) {
        case let .new(wallet): try .new(Wallet(wallet))
        case let .existing(wallet): try .existing(Wallet(wallet))
        }
    }

    func delete(_ wallet: Wallet) async throws -> GemWalletDeletion {
        try await deleteWallet(walletId: wallet.id.id)
    }

    func setup(chains: [Chain]) async throws {
        _ = try await setupChains(chains: chains.map(\.rawValue))
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
}
