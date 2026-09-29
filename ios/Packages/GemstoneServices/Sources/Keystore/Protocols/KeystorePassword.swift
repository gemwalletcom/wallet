// Copyright (c). Gem Wallet. All rights reserved.

public import enum Gemstone.GemLockPeriod
import Foundation
import LocalAuthentication
import Primitives

public protocol KeystorePassword: Sendable {
    func getPassword(createIfMissing: Bool) throws -> String
    func getAuthentication() throws -> KeystoreAuthentication
    func getAvailableAuthentication() -> KeystoreAuthentication
    func enableAuthentication(_ enable: Bool, context: LAContext) throws
    func remove() throws

    func getPrivacyLockStatus() throws -> PrivacyLockStatus?
    func setPrivacyLockStatus(_ status: PrivacyLockStatus) throws

    func getAuthenticationLockPeriod() throws -> GemLockPeriod?
    func setAuthenticationLockPeriod(period: GemLockPeriod) throws
}

public extension KeystorePassword {
    func getPassword() throws -> String {
        try getPassword(createIfMissing: false)
    }
}
