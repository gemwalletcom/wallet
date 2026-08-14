// Copyright (c). Gem Wallet. All rights reserved.

import AvatarService
import Foundation
import Keystore
import Preferences
import Primitives
import Store
import WalletSessionService

public struct WalletService: Sendable {
    private let keystore: any Keystore
    private let walletStore: WalletStore
    private let avatarService: AvatarService
    private let walletSessionService: any WalletSessionManageable
    private let preferences: ObservablePreferences

    public init(
        keystore: any Keystore,
        walletStore: WalletStore,
        preferences: ObservablePreferences,
        avatarService: AvatarService,
        walletSessionService: any WalletSessionManageable,
    ) {
        self.keystore = keystore
        self.walletStore = walletStore
        self.avatarService = avatarService
        self.walletSessionService = walletSessionService
        self.preferences = preferences
    }

    public var isAcceptTermsCompleted: Bool {
        preferences.isAcceptTermsCompleted
    }

    public func nextWalletIndex() throws -> Int {
        try walletStore.nextWalletIndex()
    }

    public func acceptTerms() {
        preferences.isAcceptTermsCompleted = true
    }

    public func createWallet() throws -> [String] {
        try keystore.createWallet()
    }

    public func loadOrCreateWallet(name: String, type: KeystoreImportType, source: WalletSource) async throws -> WalletImportResult {
        if let existing = try await existingWallet(type: type) {
            return .existing(existing)
        }
        let wallet = try await keystore.importWallet(
            name: name,
            type: type,
            isWalletsEmpty: walletSessionService.wallets.isEmpty,
            source: source,
        )
        try walletStore.addWallet(wallet)
        preferences.invalidateSubscriptions()
        return .new(wallet)
    }

    private func existingWallet(type: KeystoreImportType) async throws -> Wallet? {
        let preview = try await keystore.previewImport(type: type)
        return walletSessionService.wallets.first { wallet in
            wallet.id == preview.walletId && wallet.type == preview.walletType
        }
    }

    public func delete(_ wallet: Wallet) async throws {
        try await keystore.deleteKey(for: wallet)
        try walletStore.deleteWallet(for: wallet.id)
        try avatarService.remove(for: wallet)
        WalletPreferences(walletId: wallet.id).clear()

        await MainActor.run {
            if walletSessionService.currentWalletId == wallet.id {
                walletSessionService.setCurrent(walletId: walletSessionService.wallets.first?.id)
            }
        }

        if walletSessionService.wallets.isEmpty {
            preferences.preferences.clear()
            preferences.preferences.subscriptionsVersionHasChange = false
        }

        preferences.invalidateSubscriptions()
    }

    public func deleteEmptyWallets() async throws {
        for wallet in try walletStore.getWallets() where wallet.accounts.isEmpty {
            try await delete(wallet)
        }
    }

    public func setup(chains: [Chain]) throws {
        let wallets = walletSessionService.wallets.filter { $0.type == .multicoin }
        guard !wallets.isEmpty else { return }

        let setupWallets = try keystore.setupChains(chains: chains, for: wallets)
        for wallet in setupWallets {
            try walletStore.addWallet(wallet)
        }
        if setupWallets.isNotEmpty {
            preferences.invalidateSubscriptions()
        }
    }

    public func migrateV3Keystores() async throws {
        let wallets = try walletStore.getWallets()
        let failures = try await keystore.migrateV3Keystores(for: wallets)
        for failure in failures {
            debugLog("v3 keystore migration failed for \(failure.walletId.id): \(failure.error)")
        }
    }

    public func pin(wallet: Wallet) throws {
        try walletStore.pinWallet(wallet.id, value: true)
    }

    public func unpin(wallet: Wallet) throws {
        try walletStore.pinWallet(wallet.id, value: false)
    }

    public func rename(walletId: WalletId, newName: String) throws {
        try walletStore.renameWallet(walletId, name: newName)
    }

    public func getMnemonic(wallet: Wallet) async throws -> [String] {
        try await keystore.getMnemonic(wallet: wallet)
    }

    public func getPrivateKeyEncoded(wallet: Primitives.Wallet, chain: Chain) async throws -> String {
        try await keystore.getPrivateKeyEncoded(wallet: wallet, chain: chain)
    }
}
