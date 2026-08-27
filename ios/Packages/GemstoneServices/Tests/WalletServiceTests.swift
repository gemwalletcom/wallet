// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import Observation
import Preferences
import PreferencesTestKit
import Primitives
import Store
import StoreTestKit
import Testing
@testable import GemstoneServices

struct WalletServiceTests {
    @Test
    func deleteLastWalletNotifiesObservers() async throws {
        let preferences = ObservablePreferences.mock()
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum]))
        let walletSessionService = WalletSessionService.mock(store: walletStore, preferences: preferences)
        let service = WalletService.mock(walletStore: walletStore, preferences: preferences)

        let wallet = try await service.importWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import,
        ).wallet
        try await walletSessionService.setCurrent(wallet: wallet)

        try await confirmation { confirm in
            withObservationTracking {
                _ = preferences.currentWalletId
            } onChange: {
                confirm()
            }
            try await service.delete(wallet)
        }
    }

    @Test
    func loadOrCreateWalletMarksSubscriptionsDirty() async throws {
        let rawPreferences = Preferences.mock()
        rawPreferences.subscriptionsVersion = 4

        let service = WalletService.mock(
            walletStore: .mock(db: .mockWithChains([.ethereum])),
            preferences: .mock(preferences: rawPreferences),
        )

        _ = try await service.importWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import,
        )

        #expect(rawPreferences.subscriptionsVersion == 5)
    }

    @Test
    func deleteWalletMarksSubscriptionsDirty() async throws {
        let rawPreferences = Preferences.mock()
        let service = WalletService.mock(
            walletStore: .mock(db: .mockWithChains([.ethereum])),
            preferences: .mock(preferences: rawPreferences),
        )

        let wallet = try await service.importWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import,
        ).wallet
        _ = try await service.importWallet(
            name: "Second Wallet",
            type: .phrase(words: service.createWallet(), chains: [.ethereum]),
            source: .import,
        )

        rawPreferences.subscriptionsVersion = 7

        try await service.delete(wallet)

        #expect(rawPreferences.subscriptionsVersion == 8)
    }

    @Test
    func deleteLastWalletMarksSubscriptionsDirty() async throws {
        let rawPreferences = Preferences.mock()
        let service = WalletService.mock(
            walletStore: .mock(db: .mockWithChains([.ethereum])),
            preferences: .mock(preferences: rawPreferences),
        )

        let wallet = try await service.importWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import,
        ).wallet

        rawPreferences.subscriptionsVersion = 7

        try await service.delete(wallet)

        #expect(rawPreferences.subscriptionsVersion == 1)
    }

    @Test
    func setupChainsMarksSubscriptionsDirty() async throws {
        let rawPreferences = Preferences.mock()
        let service = WalletService.mock(
            walletStore: .mock(db: .mockWithChains([.ethereum, .bitcoin])),
            preferences: .mock(preferences: rawPreferences),
        )

        _ = try await service.importWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import,
        )

        rawPreferences.subscriptionsVersion = 10

        try await service.setup(chains: [.bitcoin])

        #expect(rawPreferences.subscriptionsVersion == 11)
    }

    @Test
    func setupChainsFailsWithoutSeededAsset() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore.mock(db: db)
        let assetStore = AssetStore.mock(db: db)
        let service = WalletService.mock(walletStore: walletStore)

        _ = try await service.importWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import,
        )

        await #expect(throws: Error.self) {
            try await service.setup(chains: [.ethereum, .seiEvm])
        }

        try assetStore.add(assets: [.mock(asset: .mock(id: AssetId(chain: .seiEvm)))])

        try await service.setup(chains: [.ethereum, .seiEvm])

        let wallet = try #require(try walletStore.getWallets().first)
        #expect(wallet.accounts.contains(where: { $0.chain == .seiEvm }))
    }

    @Test
    func passwordCreatedOnFirstImport() async throws {
        let mockPassword = MockKeystorePassword()
        let service = WalletService.mock(keystore: LocalKeystore.mock(keystorePassword: mockPassword), walletStore: .mock(db: .mockWithChains([.ethereum])))

        #expect(try mockPassword.getPassword().isEmpty)

        _ = try await service.importWallet(
            name: "First Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import,
        )

        #expect(try mockPassword.getPassword().count == 64)
    }

    @Test
    func setupChainsAddsMissingChains() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum, .solana])))
        _ = try await service.importWallet(name: "ETH only", type: .phrase(words: LocalKeystore.words, chains: [.ethereum]), source: .import)
        let store = WalletStore.mock(db: .mockWithChains([.ethereum, .solana]))
        _ = store

        try await service.setup(chains: [.ethereum, .solana])

        let wallet = try #require(try service.mockWallets().first)
        #expect(wallet.accounts.map(\.chain).asSet() == [Chain.ethereum, .solana].asSet())
    }

    @Test
    func setupChainsSkipsWalletsWithoutKeystoreWithoutReadingPassword() async throws {
        let mockPassword = MockKeystorePassword()
        let keystore = LocalKeystore.mock(keystorePassword: mockPassword)
        let service = WalletService.mock(keystore: keystore, walletStore: .mock(db: .mockWithChains([.ethereum, .solana])))
        let wallet = try await service.importWallet(name: "ETH only", type: .phrase(words: LocalKeystore.words, chains: [.ethereum]), source: .import).wallet
        try await keystore.deleteKey(for: wallet)
        let passwordReadsBefore = mockPassword.getPasswordCallsCount

        try await service.setup(chains: [.ethereum, .solana])

        #expect(try service.mockWallets().first?.accounts.count == 1)
        #expect(mockPassword.getPasswordCallsCount == passwordReadsBefore)
    }

    @Test
    func setupChainsAddNoMissingChains() async throws {
        let mockPassword = MockKeystorePassword()
        let keystore = LocalKeystore.mock(keystorePassword: mockPassword)
        let service = WalletService.mock(keystore: keystore, walletStore: .mock(db: .mockWithChains([.ethereum, .solana])))
        _ = try await service.importWallet(name: "Complete", type: .phrase(words: LocalKeystore.words, chains: [.ethereum, .solana]), source: .import)
        let passwordReadsBefore = mockPassword.getPasswordCallsCount

        try await service.setup(chains: [.ethereum, .solana])

        #expect(mockPassword.getPasswordCallsCount == passwordReadsBefore)
    }

    @Test
    func concurrentImportAndDelete() async throws {
        let service = WalletService.mock(
            keystore: LocalKeystore.mock(keystorePassword: MockKeystorePassword(memoryPassword: LocalKeystore.password)),
            walletStore: .mock(db: .mockWithChains([.ethereum])),
        )
        let words = try (0 ..< 5).map { _ in try service.createWallet() }

        let wallets = try await withThrowingTaskGroup(of: Primitives.Wallet.self) { group in
            for (index, words) in words.enumerated() {
                group.addTask {
                    try await service.importWallet(name: "Wallet \(index)", type: .phrase(words: words, chains: [.ethereum]), source: .import).wallet
                }
            }
            var wallets: [Primitives.Wallet] = []
            for try await wallet in group {
                wallets.append(wallet)
            }
            return wallets
        }
        #expect(wallets.count == 5)

        try await withThrowingTaskGroup(of: Void.self) { group in
            for wallet in wallets {
                group.addTask { try await service.delete(wallet) }
            }
            try await group.waitForAll()
        }
        #expect(try service.mockWallets().isEmpty)
    }
}
