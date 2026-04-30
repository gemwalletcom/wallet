// Copyright (c). Gem Wallet. All rights reserved.

import Keystore
import KeystoreTestKit
import NameServiceTestKit
@testable import Onboarding
import Primitives
import PrimitivesTestKit
import StoreTestKit
import Testing
import WalletService
import WalletServiceTestKit

@MainActor
struct ImportWalletSceneViewModelTests {
    @Test
    func existingImportSetsCurrentWallet() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum])))

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
        try await service.setCurrent(wallet: walletB)

        #expect(service.currentWalletId == walletB.walletId)

        let model = ImportWalletSceneViewModel(
            walletService: service,
            nameService: MockNameService(),
            type: .chain(.ethereum),
            onComplete: nil,
        )
        model.input = LocalKeystore.words.joined(separator: " ")
        await model.onSelectActionButton()

        #expect(service.currentWalletId == walletA.walletId)
    }
}
