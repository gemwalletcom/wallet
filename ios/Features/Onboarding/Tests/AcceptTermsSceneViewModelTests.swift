// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
import Primitives
import Style
import Testing

struct AcceptTermsSceneViewModelTests {
    @Test
    func theTermsComeFromCoreAndStartUnconfirmed() {
        let model = AcceptTermsSceneViewModel(preferences: .mock(), onNext: nil)

        #expect(model.viewState.rows.isNotEmpty)
        #expect(model.viewState.isAccepted == false)
        #expect(model.buttonState == .disabled)
    }

    @Test
    func continuingRecordsTheAcceptance() {
        let preferences = ObservablePreferences.mock()
        let model = AcceptTermsSceneViewModel(preferences: preferences, onNext: nil)

        #expect(preferences.isAcceptTermsCompleted == false)

        model.accept()

        #expect(preferences.isAcceptTermsCompleted)
    }

    @Test
    func everyTermMustBeTickedBeforeContinuing() {
        let model = AcceptTermsSceneViewModel(preferences: .mock(), onNext: nil)

        for row in model.viewState.rows.dropLast() {
            model.onToggle(row.item)
        }
        #expect(model.viewState.isAccepted == false)

        model.viewState.rows.filter { !$0.isAccepted }.forEach { model.onToggle($0.item) }
        #expect(model.viewState.isAccepted)
        #expect(model.buttonState == .normal)
    }
}
