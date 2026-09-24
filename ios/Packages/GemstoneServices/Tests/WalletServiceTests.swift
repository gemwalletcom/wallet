// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPreferencesService
import struct Gemstone.GemWalletImportRequest
import class Gemstone.GemWalletPreferencesService
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import GemstonePrimitives
import GemstonePrimitivesTestKit
@testable import GemstoneServices
import GemstoneServicesTestKit
import Observation
import Primitives
import Store
import StoreTestKit
import Testing

private func importRequest(words: [String] = LocalKeystore.words, name: String = "Wallet") -> GemWalletImportRequest {
    GemWalletImportRequest(
        kind: .phrase,
        chain: Primitives.Chain.ethereum.toGem(),
        input: words.joined(separator: " "),
        nameRecord: .none,
        defaultName: name,
        source: Primitives.WalletSource.import.toGem(),
    )
}

struct WalletServiceTests {
    @Test
    func deleteLastWalletNotifiesObservers() async throws {
        let sessionStore = GemstoneWalletSessionStore.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore.mock(db: db)
        let session = GemWalletSessionService.mock(store: walletStore, sessionStore: sessionStore)
        let service = GemWalletService.mock(db: db, sessionStore: sessionStore)

        let wallet = try await service.importWallet(request: importRequest()).wallet().toPrimitives()
        try session.setCurrent(walletId: wallet.id)

        try await confirmation { confirm in
            withObservationTracking {
                _ = sessionStore.currentWalletId
            } onChange: {
                confirm()
            }
            _ = try await service.delete(wallet)
        }
    }

    @Test
    func passwordCreatedOnFirstImport() async throws {
        let mockPassword = MockKeystorePassword()
        let service = GemWalletService.mock(keystore: LocalKeystore.mock(keystorePassword: mockPassword), db: .mockWithChains([.ethereum]))

        #expect(try mockPassword.getPassword().isEmpty)

        _ = try await service.importWallet(request: importRequest(name: "First Wallet"))

        #expect(try mockPassword.getPassword().count == 64)
    }

    @Test
    func concurrentImportAndDelete() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore.mock(db: db)
        let service = GemWalletService.mock(
            keystore: LocalKeystore.mock(keystorePassword: MockKeystorePassword(memoryPassword: LocalKeystore.password)),
            db: db,
        )
        let words = try (0 ..< 5).map { _ in try service.createWallet() }

        let wallets = try await withThrowingTaskGroup(of: Primitives.Wallet.self) { group in
            for (index, words) in words.enumerated() {
                group.addTask {
                    try await service.importWallet(request: importRequest(words: words, name: "Wallet \(index)")).wallet().toPrimitives()
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
                group.addTask { _ = try await service.delete(wallet) }
            }
            try await group.waitForAll()
        }
        #expect(try walletStore.getWallets().isEmpty)
    }
}
