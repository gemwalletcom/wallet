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
}
