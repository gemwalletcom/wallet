// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import GemstonePrimitives
import GemstoneServices
import GemstoneServicesTestKit
import protocol Gemstone.GemNameServiceProtocol
import enum Gemstone.GemWalletImportKind
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
@testable import Onboarding
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
        let session = GemWalletSessionService(store: sessionStore, wallets: GemstoneWalletStore(store: WalletStore.mock(db: db)))
        let service = GemWalletService.mock(db: db, sessionStore: sessionStore)

        let walletA = try await service.importWallet(
            name: "Wallet A",
            type: .singlePhrase(words: LocalKeystore.words, chain: Primitives.Chain.ethereum.map()),
            source: .import,
        ).wallet

        let walletB = try await service.importWallet(
            name: "Wallet B",
            type: .singlePhrase(words: service.createWallet(), chain: Primitives.Chain.ethereum.map()),
            source: .import,
        ).wallet
        try await session.setCurrent(wallet: walletB)

        #expect(session.currentWalletId == walletB.id)

        let model = ImportWalletSceneViewModel.mock(
            service: service,
        )
        model.input = LocalKeystore.words.joined(separator: " ")
        await model.onSelectActionButton()

        #expect(session.currentWalletId == walletA.id)
    }

    @Test
    func resolvesNameOnlyForAddressImport() async throws {
        let nameService = GemNameServiceMock(nameRecord: .mock())
        let model = ImportWalletSceneViewModel.mock(nameService: nameService)

        try await enterName(in: model, importType: .privateKey)

        #expect(nameService.requestedNames.isEmpty)

        try await enterName(in: model, importType: .address)

        #expect(nameService.requestedNames == ["vitalik.eth"])
    }

    private func enterName(in model: ImportWalletSceneViewModel, importType: GemWalletImportKind) async throws {
        model.importType = importType
        model.onChangeInput("", newValue: "vitalik.eth")
        try await Task.sleep(for: .milliseconds(500))
    }
}

@MainActor
private extension ImportWalletSceneViewModel {
    static func mock(
        service: GemWalletService? = nil,
        nameService: any GemNameServiceProtocol = GemNameServiceMock(nameRecord: .mock()),
    ) -> ImportWalletSceneViewModel {
        ImportWalletSceneViewModel(
            service: service ?? .mock(),
            preferences: .mock(),
            nameService: nameService,
            type: .chain(.ethereum),
            onComplete: nil,
        )
    }
}
