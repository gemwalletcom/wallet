// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
import Primitives
import Testing

struct AcceptTermsSceneViewModelTests {
    @Test
    func theTermsComeFromCoreAndStartUnconfirmed() {
        let model = AcceptTermsSceneViewModel(preferences: .mock(), onNext: nil)

        #expect(model.items.isNotEmpty)
        #expect(model.isConfirmed == false)
        #expect(model.state.isNoData)
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

        for item in model.items.dropLast() {
            item.isConfirmed = true
        }
        #expect(model.isConfirmed == false)

        model.items.forEach { $0.isConfirmed = true }
        #expect(model.isConfirmed)
        #expect(model.state.isNoData == false)
    }
}
