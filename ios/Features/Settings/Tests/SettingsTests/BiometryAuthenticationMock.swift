// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import LocalAuthentication
import Primitives

final class BiometryAuthenticationMock: BiometryAuthenticatable, @unchecked Sendable {
    var requiresAuthentication: Bool
    var availableAuthentication: KeystoreAuthentication
    var lockPeriod: LockPeriod
    var isPrivacyLockEnabled: Bool

    var enableError: Error?
    var privacyLockError: Error?
    var lockPeriodError: Error?

    private(set) var enableCalls: [Bool] = []
    private(set) var privacyLockCalls: [Bool] = []
    private(set) var lockPeriodCalls: [LockPeriod] = []

    init(
        requiresAuthentication: Bool = false,
        availableAuthentication: KeystoreAuthentication = .biometrics,
        lockPeriod: LockPeriod = .default,
        isPrivacyLockEnabled: Bool = false,
    ) {
        self.requiresAuthentication = requiresAuthentication
        self.availableAuthentication = availableAuthentication
        self.lockPeriod = lockPeriod
        self.isPrivacyLockEnabled = isPrivacyLockEnabled
    }

    func shouldRelock(elapsedMilliseconds _: Int64) -> Bool { requiresAuthentication }

    @MainActor
    func authenticate(context _: LAContext, reason _: String) async throws {}

    @MainActor
    func enableAuthentication(_ enable: Bool, context _: LAContext, reason _: String) async throws {
        enableCalls.append(enable)
        if let enableError { throw enableError }
        requiresAuthentication = enable
        if !enable {
            isPrivacyLockEnabled = false
            lockPeriod = .default
        }
    }

    func update(period: LockPeriod) throws {
        lockPeriodCalls.append(period)
        if let lockPeriodError { throw lockPeriodError }
        lockPeriod = period
    }

    func togglePrivacyLock(enabled: Bool) throws {
        privacyLockCalls.append(enabled)
        if let privacyLockError { throw privacyLockError }
        isPrivacyLockEnabled = enabled
    }
}
