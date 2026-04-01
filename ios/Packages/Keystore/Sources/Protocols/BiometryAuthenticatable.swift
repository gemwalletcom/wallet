// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import LocalAuthentication

public protocol BiometryAuthenticatable: Sendable {
    var isAuthenticationEnabled: Bool { get }
    var availableAuthentication: KeystoreAuthentication { get }
    var lockPeriod: LockPeriod { get }
    var isPrivacyLockEnabled: Bool { get }

    func authenticate(context: LAContext, reason: String) async throws
    func enableAuthentication(_ enable: Bool, context: LAContext, reason: String) async throws
    func update(period: LockPeriod) throws
    func togglePrivacyLock(enbaled: Bool) throws
}

public extension BiometryAuthenticatable {
    func authenticate(reason: String) async throws {
        try await authenticate(context: LAContext(), reason: reason)
    }

    func enableAuthentication(_ enable: Bool, reason: String) async throws {
        try await enableAuthentication(enable, context: LAContext(), reason: reason)
    }
}
