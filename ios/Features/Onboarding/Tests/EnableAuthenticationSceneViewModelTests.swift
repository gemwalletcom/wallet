// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
import Primitives
import Testing

@MainActor
struct EnableAuthenticationSceneViewModelTests {
    @Test
    func enablingTurnsOnAuthenticationAndFinishes() async {
        let service = BiometryAuthenticationMock(requiresAuthentication: false)
        let preferences = ObservablePreferences.mock()
        var completed = false
        let model = EnableAuthenticationSceneViewModel(service: service, preferences: preferences, onComplete: { completed = true })

        await model.enable()

        #expect(service.enableCalls == [true])
        #expect(completed)
        #expect(preferences.shouldOfferAuthentication(service: BiometryAuthenticationMock(requiresAuthentication: false)) == false)
    }

    @Test
    func skippingFinishesWithoutEnabling() {
        let service = BiometryAuthenticationMock(requiresAuthentication: false)
        let preferences = ObservablePreferences.mock()
        var completed = false
        let model = EnableAuthenticationSceneViewModel(service: service, preferences: preferences, onComplete: { completed = true })

        model.skip()

        #expect(service.enableCalls.isEmpty)
        #expect(completed)
        #expect(preferences.shouldOfferAuthentication(service: BiometryAuthenticationMock(requiresAuthentication: false)) == false)
    }

    @Test
    func aCancelledPromptKeepsTheOfferOpen() async {
        let service = BiometryAuthenticationMock(requiresAuthentication: false)
        service.enableError = BiometryAuthenticationError.cancelledByUser
        let preferences = ObservablePreferences.mock()
        var completed = false
        let model = EnableAuthenticationSceneViewModel(service: service, preferences: preferences, onComplete: { completed = true })

        await model.enable()

        #expect(completed == false)
        #expect(model.isPresentingAlertMessage == nil)
        #expect(preferences.shouldOfferAuthentication(service: BiometryAuthenticationMock(requiresAuthentication: false)))
    }
}
