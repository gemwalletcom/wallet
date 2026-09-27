// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import LocalAuthentication

public protocol Keychain: Sendable {
    func accessibility(_ accessibility: Accessibility, authenticationPolicy: AuthenticationPolicy) -> Keychain
    func authenticationContext(_ authenticationContext: LAContext) -> Keychain

    func get(_ key: String) throws -> String?
    func getData(_ key: String) throws -> Data?

    func set(_ value: String, key: String) throws
    func set(_ value: Data, key: String) throws

    func remove(_ key: String) throws
}
