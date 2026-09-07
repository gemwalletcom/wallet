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
        db: DB = .mock(),
        sessionStore: GemstoneWalletSessionStore = .mock(),
    ) -> GemWalletService {
        GemWalletService.mock(keystore: keystore, db: db, sessionStore: sessionStore)
    }

    @Test
    func deleteLastWalletNotifiesObservers() async throws {
        let sessionStore = GemstoneWalletSessionStore.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore.mock(db: db)
        let session = GemWalletSessionService(store: sessionStore, wallets: GemstoneWalletStore(store: walletStore))
        let service = makeService(db: db, sessionStore: sessionStore)

        let wallet = try await service.importWallet(
            name: "Wallet",
            type: .multicoinPhrase(words: LocalKeystore.words, chains: [Primitives.Chain.ethereum].map { $0.map() }),
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
        let service = makeService(db: db)

        _ = try await service.importWallet(
            name: "Wallet",
            type: .multicoinPhrase(words: LocalKeystore.words, chains: [Primitives.Chain.ethereum].map { $0.map() }),
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
        let service = makeService(keystore: LocalKeystore.mock(keystorePassword: mockPassword), db: .mockWithChains([.ethereum]))

        #expect(try mockPassword.getPassword().isEmpty)

        _ = try await service.importWallet(
            name: "First Wallet",
            type: .multicoinPhrase(words: LocalKeystore.words, chains: [Primitives.Chain.ethereum].map { $0.map() }),
            source: .import,
        )

        #expect(try mockPassword.getPassword().count == 64)
    }

    @Test
    func setupChainsAddsMissingChains() async throws {
        let db = DB.mockWithChains([.ethereum, .solana])
        let walletStore = WalletStore.mock(db: db)
        let service = makeService(db: db)
        _ = try await service.importWallet(name: "ETH only", type: .multicoinPhrase(words: LocalKeystore.words, chains: [Primitives.Chain.ethereum].map { $0.map() }), source: .import)

        try await service.setup(chains: [.ethereum, .solana])

        let wallet = try #require(try walletStore.getWallets().first)
        #expect(wallet.accounts.map(\.chain).asSet() == [Chain.ethereum, .solana].asSet())
    }

    @Test
    func setupChainsSkipsWalletsWithoutKeystoreWithoutReadingPassword() async throws {
        let mockPassword = MockKeystorePassword()
        let keystore = LocalKeystore.mock(keystorePassword: mockPassword)
        let db = DB.mockWithChains([.ethereum, .solana])
        let walletStore = WalletStore.mock(db: db)
        let service = makeService(keystore: keystore, db: db)
        let wallet = try await service.importWallet(name: "ETH only", type: .multicoinPhrase(words: LocalKeystore.words, chains: [Primitives.Chain.ethereum].map { $0.map() }), source: .import).wallet
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
        let service = makeService(keystore: keystore, db: .mockWithChains([.ethereum, .solana]))
        _ = try await service.importWallet(name: "Complete", type: .multicoinPhrase(words: LocalKeystore.words, chains: [Primitives.Chain.ethereum, .solana].map { $0.map() }), source: .import)
        let passwordReadsBefore = mockPassword.getPasswordCallsCount

        try await service.setup(chains: [.ethereum, .solana])

        #expect(mockPassword.getPasswordCallsCount == passwordReadsBefore)
    }

    @Test
    func concurrentImportAndDelete() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore.mock(db: db)
        let service = makeService(
            keystore: LocalKeystore.mock(keystorePassword: MockKeystorePassword(memoryPassword: LocalKeystore.password)),
            db: db,
        )
        let words = try (0 ..< 5).map { _ in try service.createWallet() }

        let wallets = try await withThrowingTaskGroup(of: Primitives.Wallet.self) { group in
            for (index, words) in words.enumerated() {
                group.addTask {
                    try await service.importWallet(name: "Wallet \(index)", type: .multicoinPhrase(words: words, chains: [Primitives.Chain.ethereum].map { $0.map() }), source: .import).wallet
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
