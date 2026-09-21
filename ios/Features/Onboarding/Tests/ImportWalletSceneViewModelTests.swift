// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
@testable import OnboardingTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

@MainActor
struct ImportWalletSceneViewModelTests {
    @Test
    func existingImportSetsCurrentWallet() async throws {
        let db = DB.mockWithChains([.ethereum])
        let sessionStore = GemstoneWalletSessionStore.mock()
        let session = GemWalletSessionService.mock(store: WalletStore.mock(db: db), sessionStore: sessionStore)
        let service = GemWalletService.mock(db: db, sessionStore: sessionStore)

        let walletA = try await service.importWallet(
            name: "Wallet A",
            type: .singlePhrase(words: LocalKeystore.words, chain: Primitives.Chain.ethereum.toGem()),
            source: .import,
        ).wallet

        let walletB = try await service.importWallet(
            name: "Wallet B",
            type: .singlePhrase(words: service.createWallet(), chain: Primitives.Chain.ethereum.toGem()),
            source: .import,
        ).wallet
        try await session.setCurrent(wallet: walletB)

        #expect(session.currentWalletId == walletB.id)

        let model = ImportWalletSceneViewModel.mock(service: service)
        model.input = LocalKeystore.words.joined(separator: " ")
        await model.onSelectActionButton()

        #expect(session.currentWalletId == walletA.id)
    }

    @Test
    func resolvesNameOnlyForAddressImport() {
        let model = ImportWalletSceneViewModel.mock()

        model.importType = .privateKey
        model.onChangeInput("", newValue: "vitalik.eth")

        #expect(model.nameRecordViewModel?.isResolving == false)

        model.importType = .address
        model.onChangeInput("", newValue: "vitalik.eth")

        #expect(model.nameRecordViewModel?.isResolving == true)
    }
}
