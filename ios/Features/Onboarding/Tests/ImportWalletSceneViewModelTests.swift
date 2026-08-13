// Copyright (c). Gem Wallet. All rights reserved.

import Keystore
import KeystoreTestKit
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
        let sessionService = walletSessionService ?? WalletSessionService.mock(store: walletStore)
        return ImportWalletSceneViewModel(
            walletService: walletService ?? .mock(walletStore: walletStore, walletSessionService: sessionService),
            walletSessionService: sessionService,
            nameService: nameService,
            type: .chain(.ethereum),
            onComplete: nil,
        )
    }
}
