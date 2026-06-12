// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Keystore
import KeystoreTestKit
import Primitives
import Testing

struct BiometryAuthenticationServiceTests {
    @Test
    func requiresAuthenticationWhenKeychainUnreadable() {
        let keystorePassword = MockKeystorePassword(availableAuthentication: .none)
        let service = BiometryAuthenticationService(keystorePassword: keystorePassword)

        #expect(!service.requiresAuthentication)

        keystorePassword.getAuthenticationError = AnyError("keychain interaction not allowed")

        #expect(service.requiresAuthentication)
    }

    @Test
    func requiresAuthenticationReflectsStoredAuthentication() {
        let keystorePassword = MockKeystorePassword(availableAuthentication: .biometrics)
        let service = BiometryAuthenticationService(keystorePassword: keystorePassword)

        #expect(service.requiresAuthentication)
    }
}
