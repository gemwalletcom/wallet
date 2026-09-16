// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import Testing
@testable import Settings

@MainActor
struct SecurityViewModelTests {
    private func model(
        service: BiometryAuthenticationMock = BiometryAuthenticationMock(),
        settings: GemSettingsServiceMock = GemSettingsServiceMock(),
    ) -> SecurityViewModel {
        SecurityViewModel(service: service, settings: settings, preferences: .mock())
    }

    @Test
    func theSceneStartsFromWhatTheKeystoreReports() {
        let service = BiometryAuthenticationMock(requiresAuthentication: true, lockPeriod: .oneMinute, isPrivacyLockEnabled: true)
        let model = model(service: service)

        #expect(model.isEnabled)
        #expect(model.isPrivacyLockEnabled)
        #expect(model.lockPeriod == .oneMinute)
    }

    @Test
    func theSectionsFollowWhetherAuthenticationIsOn() {
        let settings = GemSettingsServiceMock()
        let model = model(service: BiometryAuthenticationMock(requiresAuthentication: true), settings: settings)

        _ = model.sections

        #expect(settings.securitySectionsCalls == [true])
    }

    @Test
    func turningBiometricsOnAsksTheKeystoreOnce() async {
        let service = BiometryAuthenticationMock(requiresAuthentication: false)
        let model = model(service: service)
        model.isEnabled = true

        await model.toggleBiometrics()

        #expect(service.enableCalls == [true])
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aToggleThatMatchesTheKeystoreDoesNothing() async {
        let service = BiometryAuthenticationMock(requiresAuthentication: true)
        let model = model(service: service)

        await model.toggleBiometrics()

        #expect(service.enableCalls.isEmpty)
    }

    @Test
    func aCancelledPromptRevertsTheToggleWithoutAnAlert() async {
        let service = BiometryAuthenticationMock(requiresAuthentication: false)
        service.enableError = BiometryAuthenticationError.cancelledByUser
        let model = model(service: service)
        model.isEnabled = true

        await model.toggleBiometrics()

        #expect(model.isEnabled == false)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aFailedPromptRevertsTheToggleAndShowsTheError() async {
        let service = BiometryAuthenticationMock(requiresAuthentication: false)
        service.enableError = AnyError("keystore locked")
        let model = model(service: service)
        model.isEnabled = true

        await model.toggleBiometrics()

        #expect(model.isEnabled == false)
        #expect(model.isPresentingAlertMessage?.message == "keystore locked")
    }

    @Test
    func aFailedPrivacyLockRevertsTheToggle() {
        let service = BiometryAuthenticationMock(isPrivacyLockEnabled: false)
        service.privacyLockError = AnyError("not available")
        let model = model(service: service)
        model.isPrivacyLockEnabled = true

        model.togglePrivacyLock()

        #expect(model.isPrivacyLockEnabled == false)
        #expect(model.isPresentingAlertMessage?.message == "not available")
    }

    @Test
    func aFailedLockPeriodFallsBackToTheStoredOne() {
        let service = BiometryAuthenticationMock(lockPeriod: .oneMinute)
        service.lockPeriodError = AnyError("write failed")
        let model = model(service: service)

        model.lockPeriod = .fiveMinutes

        #expect(model.lockPeriod == .oneMinute)
        #expect(model.isPresentingAlertMessage?.message == "write failed")
    }

    @Test
    func changingTheLockPeriodStoresIt() {
        let service = BiometryAuthenticationMock(lockPeriod: .default)
        let model = model(service: service)

        model.lockPeriod = .fiveMinutes

        #expect(service.lockPeriodCalls == [.fiveMinutes])
        #expect(model.isPresentingAlertMessage == nil)
    }
}
