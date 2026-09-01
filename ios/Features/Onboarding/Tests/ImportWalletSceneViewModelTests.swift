// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import protocol Gemstone.GemNameServiceProtocol
import GemstoneServices
import GemstoneServicesTestKit
import class Gemstone.GemOnboardingService
import class Gemstone.GemWalletSessionService
import Preferences
import PreferencesTestKit
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
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum]))
        let sessionStore = GemstoneWalletSessionStore.mock()
        let session = GemWalletSessionService(store: sessionStore, wallets: GemstoneWalletStore(store: walletStore))
        let service = GemOnboardingService.mock(walletStore: walletStore, sessionStore: sessionStore)

        let walletA = try await service.importWallet(
            name: "Wallet A",
            type: .single(words: LocalKeystore.words, chain: .ethereum),
            source: .import,
        ).wallet

        let walletB = try await service.importWallet(
            name: "Wallet B",
            type: .single(words: service.createWallet(), chain: .ethereum),
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

    private func enterName(in model: ImportWalletSceneViewModel, importType: WalletImportType) async throws {
        model.importType = importType
        model.onChangeInput("", newValue: "vitalik.eth")
        try await Task.sleep(for: .milliseconds(500))
    }
}

@MainActor
private extension ImportWalletSceneViewModel {
    static func mock(
        service: GemOnboardingService? = nil,
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
