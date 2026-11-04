// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemServiceError
import struct Gemstone.GemVerifyPhraseSession
import struct Gemstone.GemVerifyPhraseSetup
@testable import Onboarding
import Primitives
import Testing

@MainActor
struct VerifyPhraseSceneViewModelTests {
    private let words = ["alpha", "beta", "gamma", "delta"]
    private let session = GemVerifyPhraseSetup(choices: ["alpha", "beta", "gamma", "delta"], session: GemVerifyPhraseSession(expected: [0, 1, 2, 3], chips: [0, 1, 2, 3], picked: [], isCreating: false))
    private let completed = GemVerifyPhraseSetup(choices: ["alpha", "beta", "gamma", "delta"], session: GemVerifyPhraseSession(expected: [0, 1, 2, 3], chips: [0, 1, 2, 3], picked: [0, 1, 2, 3], isCreating: false))

    @Test
    func failedCreationReEnablesTheButtonAndShowsTheError() async {
        let model = VerifyPhraseSceneViewModel(words: words, setup: completed) { _ in throw AnyError("keystore write failed") }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .normal)
        #expect(model.isPresentingAlertMessage?.message == AnyError("keystore write failed").localizedDescription)
    }

    @Test
    func cancelledPromptReEnablesTheButtonWithoutAnError() async {
        let model = VerifyPhraseSceneViewModel(words: words, setup: completed) { _ in throw GemServiceError.Cancelled }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .normal)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func successfulCreationKeepsTheButtonBusyWhileTheFlowMovesOn() async {
        let model = VerifyPhraseSceneViewModel(words: words, setup: completed) { _ in }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .loading(showProgress: true))
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func pickingEveryWordInOrderEnablesTheButton() {
        let model = VerifyPhraseSceneViewModel(words: words, setup: session) { _ in }

        let picks = model.groups.joined().map(model.pickWord)

        #expect(!picks.contains(false))
        #expect(model.buttonState == .normal)
        #expect(model.groups.joined().allSatisfy(model.isVerified))
        #expect(model.wordsIndex == nil)
    }

    @Test
    func aWrongWordLeavesTheButtonDisabled() {
        let model = VerifyPhraseSceneViewModel(words: words, setup: session) { _ in }
        let wrong = Array(model.groups.joined())[1]

        #expect(model.pickWord(index: wrong) == false)
        #expect(model.buttonState == .disabled)
        #expect(model.isVerified(index: wrong) == false)
        #expect(model.wordsIndex == 0)
    }

    @Test
    func verifiedRowsShowTheAppsOwnWordsUpToTheLastPick() {
        let setup = GemVerifyPhraseSetup(
            choices: ["gamma", "alpha", "delta", "beta"],
            session: GemVerifyPhraseSession(expected: [0, 1, 2, 3], chips: [2, 0, 3, 1], picked: [], isCreating: false),
        )
        let model = VerifyPhraseSceneViewModel(words: words, setup: setup) { _ in }
        let chips = Array(model.groups.joined())

        #expect(chips.map(\.word) == ["gamma", "alpha", "delta", "beta"])
        #expect(model.verifiedWords == ["", "", "", ""])

        #expect(model.pickWord(index: chips[1]))
        #expect(model.verifiedWords == ["alpha", "", "", ""])
        #expect(model.wordsIndex == 1)

        #expect([3, 0, 2].map { model.pickWord(index: chips[$0]) } == [true, true, true])
        #expect(model.verifiedWords == words)
    }
}
