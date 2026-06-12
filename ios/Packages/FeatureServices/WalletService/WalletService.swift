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
    ) {
        self.keystore = keystore
        self.walletStore = walletStore
        self.avatarService = avatarService
        walletSessionService = WalletSessionService(walletStore: walletStore, preferences: preferences)
        self.preferences = preferences
    }

    public var currentWalletId: WalletId? {
        walletSessionService.currentWalletId
    }

    public var currentWallet: Wallet? {
        walletSessionService.currentWallet
    }

    public var wallets: [Wallet] {
        walletSessionService.wallets
    }

    public var isAcceptTermsCompleted: Bool {
        preferences.isAcceptTermsCompleted
    }

    public func nextWalletIndex() throws -> Int {
        try walletStore.nextWalletIndex()
    }

    public func setCurrent(for walletId: WalletId) {
        walletSessionService.setCurrent(walletId: walletId)
    }

    public func getWallet(walletId: WalletId) throws -> Wallet {
        try walletSessionService.getWallet(walletId: walletId)
    }

    public func acceptTerms() {
        preferences.isAcceptTermsCompleted = true
    }

    public func createWallet() throws -> [String] {
        try keystore.createWallet()
    }

    @discardableResult
    public func setCurrent(wallet: Wallet) async throws -> Wallet {
        await MainActor.run {
            walletSessionService.setCurrent(walletId: wallet.id)
        }
        return wallet
    }

    public func loadOrCreateWallet(name: String, type: KeystoreImportType, source: WalletSource) async throws -> WalletImportResult {
        if let existing = try await existingWallet(type: type) {
            return .existing(existing)
        }
        let wallet = try await keystore.importWallet(
            name: name,
            type: type,
            isWalletsEmpty: wallets.isEmpty,
            source: source,
        )
        try walletStore.addWallet(wallet)
        preferences.invalidateSubscriptions()
        return .new(wallet)
    }

    private func existingWallet(type: KeystoreImportType) async throws -> Wallet? {
        let preview = try await keystore.previewImport(type: type)
        return wallets.first { wallet in
            wallet.id == preview.walletId && wallet.type == preview.walletType
        }
    }

    public func delete(_ wallet: Wallet) async throws {
        try await keystore.deleteKey(for: wallet)
        try walletStore.deleteWallet(for: wallet.id)
        try avatarService.remove(for: wallet)
        WalletPreferences(walletId: wallet.id).clear()

        await MainActor.run {
            if currentWalletId == wallet.id {
                walletSessionService.setCurrent(walletId: wallets.first?.id)
            }
        }

        if wallets.isEmpty {
            preferences.preferences.clear()
            preferences.preferences.subscriptionsVersionHasChange = false
        }

        preferences.invalidateSubscriptions()
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

    public func swapOrder(from: WalletId, to: WalletId) throws {
        try walletStore.swapOrder(from: from, to: to)
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
