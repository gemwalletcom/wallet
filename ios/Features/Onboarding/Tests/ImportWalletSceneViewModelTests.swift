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

        #expect(try session.getCurrentWalletId() == walletB.id, "an import needs no second call to become current")

        let model = ImportWalletSceneViewModel.mock(service: service)
        model.inputModel.text = LocalKeystore.words.joined(separator: " ")
        await model.onSelectActionButton()

        #expect(try session.getCurrentWalletId() == walletA.id)
    }

    @Test
    func continuingFromTheExistingWalletSheetClosesItFirst() async throws {
        let service = GemWalletService.mock(db: .mock(chains: [.ethereum]))
        let phrase = LocalKeystore.words.joined(separator: " ")
        _ = try await service.importWallet(kind: .phrase, chain: .ethereum, input: phrase, nameRecord: .none, source: .import)
        var completed = false
        let model = ImportWalletSceneViewModel.mock(service: service, onComplete: { completed = true })
        model.inputModel.text = phrase

        await model.onSelectActionButton()

        #expect(model.isPresentingExistingWallet != nil)
        #expect(completed == false)

        model.onSelectExistingWalletContinue()

        #expect(model.isPresentingExistingWallet == nil)
        #expect(completed)
    }

    @Test
    @available(iOS 26.0, *)
    func theWordsTheImportRefusesAreHighlightedUntilTheInputChanges() async {
        let model = ImportWalletSceneViewModel.mock()
        let editor = PhraseTextEditorViewModel(placeholder: "", input: model.inputModel)
        let highlighted = { editor.content.runs.filter { $0.foregroundColor != nil }.map { String(editor.content[$0.range].characters) } }
        model.inputModel.text = "Maestro abandon maestro"
        editor.onChangeText()

        await model.onSelectActionButton()
        editor.onChangeInvalidWords()
        #expect(highlighted() == ["Maestro", "maestro"])

        model.inputModel.text = "Maestro abandon"
        editor.onChangeText()
        #expect(highlighted().isEmpty)
    }

    @Test
    func resolvesNameOnlyForAddressImport() {
        let model = ImportWalletSceneViewModel.mock()

        model.importType = .privateKey
        model.onChangeInput("", newValue: "vitalik.eth")

        #expect(model.nameRecordViewModel.state.indicator() != .loading)

        model.importType = .address
        model.onChangeInput("", newValue: "vitalik.eth")

        #expect(model.nameRecordViewModel.state.indicator() == .loading)
    }

    @Test
    func aSuggestionCompletesTheLastWord() {
        let model = ImportWalletSceneViewModel.mock()

        model.inputModel.text = "abandon woo"

        #expect(model.wordsSuggestion == ["wood", "wool"])

        model.onSelectWord("wood")

        #expect(model.inputModel.text == "abandon wood ")
        #expect(model.inputModel.cursor == 13)
        #expect(model.wordsSuggestion.isEmpty)
    }

    @Test
    func aCursorInsideThePhraseHidesSuggestions() {
        let model = ImportWalletSceneViewModel.mock()
        model.inputModel.text = "abandon woo"

        model.inputModel.cursor = 3

        #expect(model.wordsSuggestion.isEmpty)

        model.inputModel.cursor = 11

        #expect(model.wordsSuggestion == ["wood", "wool"])
    }
}
