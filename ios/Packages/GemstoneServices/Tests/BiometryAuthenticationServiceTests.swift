// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemSecurityService
@testable import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import Testing

struct BiometryAuthenticationServiceTests {
    @Test
    func requiresAuthenticationWhenKeychainUnreadable() {
        let keystorePassword = MockKeystorePassword(availableAuthentication: .none)
        let service = BiometryAuthenticationService(keystorePassword: keystorePassword, securityService: GemSecurityService())

        #expect(!service.requiresAuthentication)

        keystorePassword.getAuthenticationError = AnyError("keychain interaction not allowed")

        #expect(service.requiresAuthentication)
    }

    @Test
    func privacyLockStaysOnWhenTheKeychainIsUnreadable() {
        let keystorePassword = MockKeystorePassword(privacyLockStatus: .none)
        let service = BiometryAuthenticationService(keystorePassword: keystorePassword, securityService: GemSecurityService())

        #expect(!service.isPrivacyLockEnabled)

        keystorePassword.getPrivacyLockStatusError = AnyError("keychain interaction not allowed")

        #expect(service.isPrivacyLockEnabled)
    }

    @Test
    func requiresAuthenticationReflectsStoredAuthentication() {
        let keystorePassword = MockKeystorePassword(availableAuthentication: .biometrics)
        let service = BiometryAuthenticationService(keystorePassword: keystorePassword, securityService: GemSecurityService())

        #expect(service.requiresAuthentication)
    }

    @Test
    @MainActor
    func authenticateIfRequiredSkipsWhenAuthenticationIsOff() async throws {
        let biometry = BiometryAuthenticationMock(requiresAuthentication: false)

        let allowed = try await biometry.authenticateIfRequired(reason: "Delete wallet")

        #expect(allowed)
        #expect(biometry.authenticateCallsCount == 0)
    }

    @Test
    @MainActor
    func authenticateIfRequiredPromptsWhenAuthenticationIsOn() async throws {
        let biometry = BiometryAuthenticationMock()

        let allowed = try await biometry.authenticateIfRequired(reason: "Delete wallet")

        #expect(allowed)
        #expect(biometry.authenticateCallsCount == 1)
    }

    @Test
    @MainActor
    func aCancelledPromptDeniesTheAction() async throws {
        let biometry = BiometryAuthenticationMock()
        biometry.authenticateError = BiometryAuthenticationError.cancelledByUser

        let allowed = try await biometry.authenticateIfRequired(reason: "Delete wallet")

        #expect(!allowed)
    }
}
