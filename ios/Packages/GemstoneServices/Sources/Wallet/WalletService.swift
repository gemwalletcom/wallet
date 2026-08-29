// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives

public struct WalletService: Sendable {
    private let service: any GemWalletServiceProtocol
    private let keystore: any Keystore
    private let walletSessionService: any WalletSessionManageable
    private let preferences: ObservablePreferences

    public init(
        service: any GemWalletServiceProtocol,
        keystore: any Keystore,
        walletSessionService: any WalletSessionManageable,
        preferences: ObservablePreferences,
    ) {
        self.service = service
        self.keystore = keystore
        self.walletSessionService = walletSessionService
        self.preferences = preferences
    }

    public var isAcceptTermsCompleted: Bool {
        preferences.isAcceptTermsCompleted
    }

    public func nextWalletIndex() throws -> Int {
        Int(try service.nextWalletIndex())
    }

    public func acceptTerms() {
        preferences.acceptTerms()
    }

    public func createWallet() throws -> [String] {
        try service.createWallet()
    }

    public func sorted(wallets: [Wallet]) -> [Wallet] {
        guard let sorted = try? service.sortedWallets(wallets: wallets.map { try $0.json() }).map({ try Wallet($0) }) else {
            return wallets
        }
        return sorted
    }

    public func importWallet(name: String, type: KeystoreImportType, source: WalletSource) async throws -> WalletImportResult {
        let walletImport = try service.validateImport(import: type.walletImport)
        return switch try await service.importWallet(name: name, import: walletImport, source: source.map()) {
        case let .new(wallet): try .new(Wallet(wallet))
        case let .existing(wallet): try .existing(Wallet(wallet))
        }
    }

    public func delete(_ wallet: Wallet) async throws {
        try await keystore.deleteKey(for: wallet)
        _ = try await service.deleteWallet(walletId: wallet.id.id)
    }


    public func setup(chains: [Chain]) async throws {
        _ = try await service.setupChains(chains: chains.map(\.rawValue))
    }

    public func migrateV3Keystores() async throws {
        let wallets = try walletSessionService.getWallets()
        let failures = try await keystore.migrateV3Keystores(for: wallets)
        for failure in failures {
            debugLog("v3 keystore migration failed for \(failure.walletId.id): \(failure.error)")
        }
    }

    public func pin(wallet: Wallet) async throws {
        try await service.setPinned(walletId: wallet.id.id, pinned: true)
    }

    public func unpin(wallet: Wallet) async throws {
        try await service.setPinned(walletId: wallet.id.id, pinned: false)
    }

    public func rename(walletId: WalletId, newName: String) async throws {
        try await service.rename(walletId: walletId.id, name: newName)
    }

    public func getMnemonic(wallet: Wallet) async throws -> [String] {
        try await keystore.getMnemonic(wallet: wallet)
    }

    public func getPrivateKeyEncoded(wallet: Primitives.Wallet, chain: Chain) async throws -> String {
        try await keystore.getPrivateKeyEncoded(wallet: wallet, chain: chain)
    }
}
