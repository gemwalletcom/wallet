// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import GemstonePrimitives
import class Gemstone.GemPreferencesService
import class Gemstone.GemWalletPreferencesService
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import Observation
import Primitives
import Store
import StoreTestKit
import GemstonePrimitivesTestKit
import Testing
@testable import GemstoneServices

struct WalletServiceTests {
    private func makeService(
        keystore: LocalKeystore = LocalKeystore.mock(),
        walletStore: WalletStore = .mock(),
        sessionStore: GemstoneWalletSessionStore = .mock(),
    ) -> GemWalletService {
        GemWalletService.mock(keystore: keystore, walletStore: walletStore, sessionStore: sessionStore)
    }

    @Test
    func deleteLastWalletNotifiesObservers() async throws {
        let sessionStore = GemstoneWalletSessionStore.mock()
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum]))
        let session = GemWalletSessionService(store: sessionStore, wallets: GemstoneWalletStore(store: walletStore))
        let service = makeService(walletStore: walletStore, sessionStore: sessionStore)

        let wallet = try await service.importWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import,
        ).wallet
        try await session.setCurrent(wallet: wallet)

        try await confirmation { confirm in
            withObservationTracking {
                _ = sessionStore.currentWalletId
            } onChange: {
                confirm()
            }
            try await service.delete(wallet)
        }
    }





    @Test
    func setupChainsFailsWithoutSeededAsset() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore.mock(db: db)
        let assetStore = AssetStore.mock(db: db)
        let service = makeService(walletStore: walletStore)

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
        let service = makeService(keystore: LocalKeystore.mock(keystorePassword: mockPassword), walletStore: .mock(db: .mockWithChains([.ethereum])))

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
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum, .solana]))
        let service = makeService(walletStore: walletStore)
        _ = try await service.importWallet(name: "ETH only", type: .phrase(words: LocalKeystore.words, chains: [.ethereum]), source: .import)
        let store = WalletStore.mock(db: .mockWithChains([.ethereum, .solana]))
        _ = store

        try await service.setup(chains: [.ethereum, .solana])

        let wallet = try #require(try walletStore.getWallets().first)
        #expect(wallet.accounts.map(\.chain).asSet() == [Chain.ethereum, .solana].asSet())
    }

    @Test
    func setupChainsSkipsWalletsWithoutKeystoreWithoutReadingPassword() async throws {
        let mockPassword = MockKeystorePassword()
        let keystore = LocalKeystore.mock(keystorePassword: mockPassword)
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum, .solana]))
        let service = makeService(keystore: keystore, walletStore: walletStore)
        let wallet = try await service.importWallet(name: "ETH only", type: .phrase(words: LocalKeystore.words, chains: [.ethereum]), source: .import).wallet
        _ = try keystore.gemKeystore.delete(keystoreId: keystore.gemKeystore.keystoreId(walletId: wallet.id.id))
        let passwordReadsBefore = mockPassword.getPasswordCallsCount

        try await service.setup(chains: [.ethereum, .solana])

        #expect(try walletStore.getWallets().first?.accounts.count == 1)
        #expect(mockPassword.getPasswordCallsCount == passwordReadsBefore)
    }

    @Test
    func setupChainsAddNoMissingChains() async throws {
        let mockPassword = MockKeystorePassword()
        let keystore = LocalKeystore.mock(keystorePassword: mockPassword)
        let service = makeService(keystore: keystore, walletStore: .mock(db: .mockWithChains([.ethereum, .solana])))
        _ = try await service.importWallet(name: "Complete", type: .phrase(words: LocalKeystore.words, chains: [.ethereum, .solana]), source: .import)
        let passwordReadsBefore = mockPassword.getPasswordCallsCount

        try await service.setup(chains: [.ethereum, .solana])

        #expect(mockPassword.getPasswordCallsCount == passwordReadsBefore)
    }

    @Test
    func concurrentImportAndDelete() async throws {
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum]))
        let service = makeService(
            keystore: LocalKeystore.mock(keystorePassword: MockKeystorePassword(memoryPassword: LocalKeystore.password)),
            walletStore: walletStore,
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
        #expect(try walletStore.getWallets().isEmpty)
    }
}
