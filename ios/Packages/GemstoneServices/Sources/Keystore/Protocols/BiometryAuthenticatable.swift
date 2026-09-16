// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAppLockSettings
import Primitives
import Foundation
import LocalAuthentication

@MainActor
public protocol BiometryAuthenticatable: AnyObject, Sendable {
    var requiresAuthentication: Bool { get }
    var availableAuthentication: KeystoreAuthentication { get }
    var lockPeriod: LockPeriod { get }
    var isPrivacyLockEnabled: Bool { get }
    var isAuthenticating: Bool { get }

    func authenticate(context: LAContext) async throws
    func enableAuthentication(_ enable: Bool, context: LAContext) async throws
    func update(period: LockPeriod) throws
    func togglePrivacyLock(enabled: Bool) throws
}

public extension BiometryAuthenticatable {
    var lockSettings: GemAppLockSettings {
        GemAppLockSettings(
            authenticationRequired: requiresAuthentication,
            privacyLockEnabled: isPrivacyLockEnabled,
            lockPeriod: lockPeriod.gemLockPeriod,
        )
    }

    func enableAuthentication(_ enable: Bool) async throws {
        try await enableAuthentication(enable, context: LAContext())
    }
}
