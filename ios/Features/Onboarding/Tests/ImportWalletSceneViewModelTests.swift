// Copyright (c). Gem Wallet. All rights reserved.

import PreferencesTestKit
import Preferences
import Keystore
import KeystoreTestKit
@testable import Onboarding
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
import GemstoneServices
import GemstoneServicesTestKit

@MainActor
struct ImportWalletSceneViewModelTests {
    @Test
    func existingImportSetsCurrentWallet() async throws {
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum]))
        let preferences = ObservablePreferences.mock()
        let walletSessionService = WalletSessionService.mock(store: walletStore, preferences: preferences)
        let service = WalletService.mock(walletStore: walletStore, preferences: preferences)

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
        await walletSessionService.setCurrent(wallet: walletB)

        #expect(walletSessionService.currentWalletId == walletB.id)

        let model = ImportWalletSceneViewModel.mock(
            walletService: service,
            walletSessionService: walletSessionService,
        )
        model.input = LocalKeystore.words.joined(separator: " ")
        await model.onSelectActionButton()

        #expect(walletSessionService.currentWalletId == walletA.id)
    }

    @Test
    func resolvesNameOnlyForAddressImport() async throws {
        let nameService = MockNameService()
        let model = ImportWalletSceneViewModel.mock(nameService: nameService)

        try await enterName(in: model, importType: .privateKey)

        #expect(await nameService.requests.isEmpty)

        try await enterName(in: model, importType: .address)

        #expect(await nameService.requests == ["vitalik.eth"])
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
        walletService: WalletService? = nil,
        walletSessionService: (any WalletSessionManageable)? = nil,
        nameService: any NameServiceable = MockNameService(),
    ) -> ImportWalletSceneViewModel {
        let walletStore = WalletStore.mock(db: .mockWithChains([.ethereum]))
        let preferences = ObservablePreferences.mock()
        let sessionService = walletSessionService ?? WalletSessionService.mock(store: walletStore, preferences: preferences)
        return ImportWalletSceneViewModel(
            walletService: walletService ?? .mock(walletStore: walletStore, preferences: preferences),
            walletSessionService: sessionService,
            nameService: nameService,
            type: .chain(.ethereum),
            onComplete: nil,
        )
    }
}
