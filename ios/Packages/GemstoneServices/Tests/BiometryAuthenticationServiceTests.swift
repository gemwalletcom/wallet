// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import Testing

@MainActor
struct BiometryAuthenticationServiceTests {
    @Test
    func requiresAuthenticationWhenKeychainUnreadable() {
        let keystorePassword = MockKeystorePassword(availableAuthentication: .none)
        let service = BiometryAuthenticationService(keystorePassword: keystorePassword, reason: "")

        #expect(!service.requiresAuthentication)

        keystorePassword.getAuthenticationError = AnyError("keychain interaction not allowed")

        #expect(service.requiresAuthentication)
    }

    @Test
    func privacyLockStaysOnWhenTheKeychainIsUnreadable() {
        let keystorePassword = MockKeystorePassword(privacyLockStatus: .none)
        let service = BiometryAuthenticationService(keystorePassword: keystorePassword, reason: "")

        #expect(!service.isPrivacyLockEnabled)

        keystorePassword.getPrivacyLockStatusError = AnyError("keychain interaction not allowed")

        #expect(service.isPrivacyLockEnabled)
    }

    @Test
    func requiresAuthenticationReflectsStoredAuthentication() {
        let keystorePassword = MockKeystorePassword(availableAuthentication: .biometrics)
        let service = BiometryAuthenticationService(keystorePassword: keystorePassword, reason: "")

        #expect(service.requiresAuthentication)
    }
}
