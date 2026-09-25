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
    func importActivatesTheWalletItStored() async throws {
        let db = DB.mock(chains: [.ethereum])
        let sessionStore = GemstoneWalletSessionStore.mock()
        let session = GemWalletSessionService.mock(store: WalletStore.mock(db: db), sessionStore: sessionStore)
        let service = GemWalletService.mock(db: db, sessionStore: sessionStore)

        let walletA = try await service.importWallet(
            kind: .phrase,
            chain: .ethereum,
            input: LocalKeystore.words.joined(separator: " "),
            nameRecord: .none,
            source: .import,
        ).wallet().toPrimitives()

        let walletB = try await service.importWallet(
            kind: .phrase,
            chain: .ethereum,
            input: service.createWallet().joined(separator: " "),
            nameRecord: .none,
            source: .import,
        ).wallet().toPrimitives()

        #expect(session.currentWalletId == walletB.id, "an import needs no second call to become current")

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

        #expect(model.nameRecordViewModel.isResolving == false)

        model.importType = .address
        model.onChangeInput("", newValue: "vitalik.eth")

        #expect(model.nameRecordViewModel.isResolving == true)
    }

    @Test
    func aSuggestionCompletesTheLastWord() {
        let model = ImportWalletSceneViewModel.mock()

        model.input = "abandon woo"

        #expect(model.wordsSuggestion == ["wood", "wool"])

        model.onSelectWord("wood")

        #expect(model.input == "abandon wood ")
        #expect(model.inputCursor == 13)
        #expect(model.wordsSuggestion.isEmpty)
    }

    @Test
    func aCursorInsideThePhraseHidesSuggestions() {
        let model = ImportWalletSceneViewModel.mock()
        model.input = "abandon woo"

        model.inputCursor = 3

        #expect(model.wordsSuggestion.isEmpty)

        model.inputCursor = 11

        #expect(model.wordsSuggestion == ["wood", "wool"])
    }
}
