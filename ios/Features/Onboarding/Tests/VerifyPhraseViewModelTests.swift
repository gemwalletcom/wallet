// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemServiceError
@testable import Onboarding
import Primitives
import Testing

@MainActor
struct VerifyPhraseViewModelTests {
    private let words = ["alpha", "beta", "gamma", "delta"]

    @Test
    func failedCreationReEnablesTheButtonAndShowsTheError() async {
        let model = VerifyPhraseViewModel(words: words) { _ in throw AnyError("keystore write failed") }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .normal)
        #expect(model.isPresentingAlertMessage?.message == AnyError("keystore write failed").localizedDescription)
    }

    @Test
    func cancelledPromptReEnablesTheButtonWithoutAnError() async {
        let model = VerifyPhraseViewModel(words: words) { _ in throw GemServiceError.Cancelled }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .normal)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func successfulCreationKeepsTheButtonBusyWhileTheFlowMovesOn() async {
        let model = VerifyPhraseViewModel(words: words) { _ in }
        model.onContinue()

        await model.complete()

        #expect(model.buttonState == .loading(showProgress: true))
        #expect(model.isPresentingAlertMessage == nil)
    }
}
