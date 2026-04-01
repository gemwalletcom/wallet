// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keychain
import LocalAuthentication

public struct KeychainMock: Keychain {
    public init() {}

    public func accessibility(_: Accessibility, authenticationPolicy _: AuthenticationPolicy) -> Keychain {
        KeychainMock()
    }

    public func authenticationContext(_: LAContext) -> Keychain {
        KeychainMock()
    }

    public func get(_: String, ignoringAttributeSynchronizable _: Bool) throws -> String? {
        nil
    }

    public func getString(_: String, ignoringAttributeSynchronizable _: Bool) throws -> String? {
        nil
    }

    public func getData(_: String, ignoringAttributeSynchronizable _: Bool) throws -> Data? {
        nil
    }

    public func set(_: String, key _: String, ignoringAttributeSynchronizable _: Bool) throws {}
    public func set(_: Data, key _: String, ignoringAttributeSynchronizable _: Bool) throws {}

    public func remove(_: String, ignoringAttributeSynchronizable _: Bool) throws {}
}
