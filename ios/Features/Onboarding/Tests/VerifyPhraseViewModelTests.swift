// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemServiceError
import struct Gemstone.GemVerifyPhraseSession
import struct Gemstone.GemVerifyPhraseSetup
@testable import Onboarding
import Primitives
import Testing

@MainActor
struct VerifyPhraseViewModelTests {
    private let words = ["alpha", "beta", "gamma", "delta"]
    private let session = GemVerifyPhraseSetup(choices: ["alpha", "beta", "gamma", "delta"], session: GemVerifyPhraseSession(expected: [0, 1, 2, 3], chips: [0, 1, 2, 3], picked: [], isCreating: false))
    private let completed = GemVerifyPhraseSetup(choices: ["alpha", "beta", "gamma", "delta"], session: GemVerifyPhraseSession(expected: [0, 1, 2, 3], chips: [0, 1, 2, 3], picked: [0, 1, 2, 3], isCreating: false))

    @Test
    func failedCreationReEnablesTheButtonAndShowsTheError() async {
        let model = VerifyPhraseViewModel(words: words, setup: completed) { _ in throw AnyError("keystore write failed") }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .normal)
        #expect(model.isPresentingAlertMessage?.message == AnyError("keystore write failed").localizedDescription)
    }

    @Test
    func cancelledPromptReEnablesTheButtonWithoutAnError() async {
        let model = VerifyPhraseViewModel(words: words, setup: completed) { _ in throw GemServiceError.Cancelled }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .normal)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func successfulCreationKeepsTheButtonBusyWhileTheFlowMovesOn() async {
        let model = VerifyPhraseViewModel(words: words, setup: completed) { _ in }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .loading(showProgress: true))
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func pickingEveryWordInOrderEnablesTheButton() {
        let model = VerifyPhraseViewModel(words: words, setup: session) { _ in }

        model.groups.joined().forEach(model.pickWord)

        #expect(model.buttonState == .normal)
        #expect(model.groups.joined().allSatisfy(model.isVerified))
        #expect(model.wordsIndex == nil)
    }

    @Test
    func aWrongWordLeavesTheButtonDisabled() {
        let model = VerifyPhraseViewModel(words: words, setup: session) { _ in }

        model.pickWord(index: Array(model.groups.joined())[1])

        #expect(model.buttonState == .disabled)
        #expect(model.isVerified(index: Array(model.groups.joined())[1]) == false)
        #expect(model.wordsIndex == 0)
    }

    @Test
    func verifiedRowsShowTheAppsOwnWordsUpToTheLastPick() {
        let model = VerifyPhraseViewModel(words: words, setup: session) { _ in }

        model.pickWord(index: Array(model.groups.joined())[0])

        #expect(model.groups.joined().map(\.word) == words)
        #expect(model.wordsIndex == 1)
    }
}
