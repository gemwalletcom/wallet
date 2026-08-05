// Copyright (c). Gem Wallet. All rights reserved.

import Keystore
import KeystoreTestKit
import NameServiceTestKit
@testable import Onboarding
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
import WalletService
import WalletServiceTestKit
import WalletSessionService
import WalletSessionServiceTestKit

@MainActor
struct ImportWalletSceneViewModelTests {
    @Test
    func existingImportSetsCurrentWallet() async throws {
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum]))
        let walletSessionService = WalletSessionService.mock(store: walletStore)
        let service = WalletService.mock(walletStore: walletStore, walletSessionService: walletSessionService)

        let walletA = try await service.loadOrCreateWallet(
            name: "Wallet A",
            type: .single(words: LocalKeystore.words, chain: .ethereum),
            source: .import,
        ).wallet

        let walletB = try await service.loadOrCreateWallet(
            name: "Wallet B",
            type: .single(words: service.createWallet(), chain: .ethereum),
            source: .import,
        ).wallet
        await walletSessionService.setCurrent(wallet: walletB)

        #expect(walletSessionService.currentWalletId == walletB.id)

        let model = ImportWalletSceneViewModel(
            walletService: service,
            walletSessionService: walletSessionService,
            nameService: MockNameService(),
            type: .chain(.ethereum),
            onComplete: nil,
        )
        model.input = LocalKeystore.words.joined(separator: " ")
        await model.onSelectActionButton()

        #expect(walletSessionService.currentWalletId == walletA.id)
    }
}
