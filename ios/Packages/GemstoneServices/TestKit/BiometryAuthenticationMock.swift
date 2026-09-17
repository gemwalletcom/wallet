// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemSecurityService
import GemstoneServices
import LocalAuthentication
import Primitives

public final class BiometryAuthenticationMock: BiometryAuthenticatable, @unchecked Sendable {
    public var requiresAuthentication: Bool
    public var availableAuthentication: KeystoreAuthentication
    public var lockPeriod: LockPeriod
    public var isPrivacyLockEnabled: Bool

    public var authenticateError: (any Error)?
    public var enableError: (any Error)?
    public var privacyLockError: (any Error)?
    public var lockPeriodError: (any Error)?
    public var holdAuthentication = false

    public private(set) var authenticateCallsCount = 0
    public private(set) var enableCalls: [Bool] = []
    public private(set) var privacyLockCalls: [Bool] = []
    public private(set) var lockPeriodCalls: [LockPeriod] = []

    private var holdContinuations: [CheckedContinuation<Void, Never>] = []

    public init(
        requiresAuthentication: Bool = true,
        availableAuthentication: KeystoreAuthentication = .biometrics,
        lockPeriod: LockPeriod = .default,
        isPrivacyLockEnabled: Bool = false,
    ) {
        self.requiresAuthentication = requiresAuthentication
        self.availableAuthentication = availableAuthentication
        self.lockPeriod = lockPeriod
        self.isPrivacyLockEnabled = isPrivacyLockEnabled
    }

    public func shouldRelock(elapsedMilliseconds: Int64) -> Bool {
        GemSecurityService().shouldRelock(
            elapsedMilliseconds: elapsedMilliseconds,
            lockIntervalMinutes: lockPeriod.gemLockPeriod.minutes(),
            authRequired: requiresAuthentication,
            hasPendingRequest: false,
        )
    }

    @MainActor
    public func authenticate(context _: LAContext, reason _: String) async throws {
        authenticateCallsCount += 1
        if holdAuthentication {
            await withCheckedContinuation { holdContinuations.append($0) }
        }
        if let authenticateError { throw authenticateError }
    }

    @MainActor
    public func enableAuthentication(_ enable: Bool, context _: LAContext, reason _: String) async throws {
        enableCalls.append(enable)
        if let enableError { throw enableError }
        requiresAuthentication = enable
        if !enable {
            isPrivacyLockEnabled = false
            lockPeriod = .default
        }
    }

    public func update(period: LockPeriod) throws {
        lockPeriodCalls.append(period)
        if let lockPeriodError { throw lockPeriodError }
        lockPeriod = period
    }

    public func togglePrivacyLock(enabled: Bool) throws {
        privacyLockCalls.append(enabled)
        if let privacyLockError { throw privacyLockError }
        isPrivacyLockEnabled = enabled
    }

    public func releaseAuthentication() {
        holdAuthentication = false
        holdContinuations.forEach { $0.resume() }
        holdContinuations.removeAll()
    }

    public func releaseNextAuthentication() {
        guard holdContinuations.isNotEmpty else { return }
        holdContinuations.removeFirst().resume()
    }
}
