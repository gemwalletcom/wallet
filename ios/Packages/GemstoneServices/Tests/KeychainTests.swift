// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Keychain
import Primitives
import Security
import Testing

struct KeychainTests {
    @Test
    func errorKeepsStatus() {
        #expect(KeychainError(status: errSecUserCanceled).isAuthenticationCancelled)
        #expect(!KeychainError(status: errSecAuthFailed).isAuthenticationCancelled)
        #expect((KeychainError(status: -1) as NSError).code == -1)
    }

    @Test
    func attributesWithoutPolicyUseAccessible() {
        let (attributes, error) = Options(accessibility: .whenUnlockedThisDeviceOnly).attributes(key: "key", value: Data())

        #expect(error == nil)
        #expect(attributes[AttributeAccessible] as? String == String(kSecAttrAccessibleWhenUnlockedThisDeviceOnly))
        #expect(attributes[AttributeAccessControl] == nil)
    }

    @Test
    func attributesWithPolicyUseAccessControl() {
        for policy: AuthenticationPolicy in [[], [.devicePasscode], [.biometryAny, .or, .devicePasscode]] {
            let (attributes, error) = Options(accessibility: .whenUnlockedThisDeviceOnly, authenticationPolicy: policy).attributes(key: "key", value: Data())

            #expect(error == nil)
            #expect(attributes[AttributeAccessible] == nil)
            #expect(attributes[AttributeAccessControl] != nil)
        }
    }
}
