// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemOnboardingServiceProtocol
import Primitives

public extension GemOnboardingServiceProtocol {
    func nextWalletIndex() throws -> Int {
        Int(try nextWalletIndex() as Int32)
    }

    func importWallet(name: String, type: KeystoreImportType, source: WalletSource) async throws -> WalletImportResult {
        let walletImport = try validateImport(import: type.walletImport)
        return switch try await importWallet(name: name, import: walletImport, source: source.map()) {
        case let .new(wallet): try .new(Wallet(wallet))
        case let .existing(wallet): try .existing(Wallet(wallet))
        }
    }

    func rename(walletId: WalletId, newName: String) async throws {
        try await rename(walletId: walletId.id, name: newName)
    }

    func setup(chains: [Chain]) async throws {
        _ = try await setupChains(chains: chains.map(\.rawValue))
    }

    func getWallets() throws -> [Wallet] {
        try wallets().map { try Wallet($0) }
    }
}
