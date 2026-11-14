// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemKeystoreAuthentication
import enum Gemstone.GemLockPeriod
import class Gemstone.GemSecurityService
import GemstoneServices
import LocalAuthentication
import Primitives

public final class BiometryAuthenticationMock: BiometryAuthenticatable, @unchecked Sendable {
    public var requiresAuthentication: Bool
    public var availableAuthentication: GemKeystoreAuthentication
    public var isPasscodeSet: Bool
    public var lockPeriod: GemLockPeriod
    public let systemPrompt = SystemPrompt()

    public var authenticateError: (any Error)?
    public var enableError: (any Error)?
    public var lockPeriodError: (any Error)?
    public var holdAuthentication = false

    public private(set) var authenticateCallsCount = 0
    public private(set) var enableCalls: [Bool] = []
    public private(set) var lockPeriodCalls: [GemLockPeriod] = []

    private var holdContinuations: [CheckedContinuation<Void, Never>] = []

    public init(
        requiresAuthentication: Bool = true,
        availableAuthentication: GemKeystoreAuthentication = .biometrics,
        isPasscodeSet: Bool = true,
        lockPeriod: GemLockPeriod = .default,
    ) {
        self.requiresAuthentication = requiresAuthentication
        self.availableAuthentication = availableAuthentication
        self.isPasscodeSet = isPasscodeSet
        self.lockPeriod = lockPeriod
    }

    public func shouldRelock(elapsedMilliseconds: Int64) -> Bool {
        GemSecurityService().shouldRelock(
            elapsedMilliseconds: elapsedMilliseconds,
            lockIntervalMinutes: lockPeriod.minutes(),
            authRequired: requiresAuthentication,
        )
    }

    public func hasPresentedSystemPrompt(since instant: ContinuousClock.Instant) -> Bool {
        systemPrompt.hasPresented(since: instant)
    }

    @MainActor
    public func authenticate(context _: LAContext, reason _: String) async throws {
        authenticateCallsCount += 1
        if holdAuthentication {
            await withCheckedContinuation { holdContinuations.append($0) }
        }
        if let authenticateError {
            throw authenticateError
        }
    }

    @MainActor
    public func enableAuthentication(_ enable: Bool, context _: LAContext, reason _: String) async throws {
        enableCalls.append(enable)
        if let enableError {
            throw enableError
        }
        requiresAuthentication = enable
        if !enable {
            lockPeriod = .default
        }
    }

    public func update(period: GemLockPeriod) throws {
        lockPeriodCalls.append(period)
        if let lockPeriodError {
            throw lockPeriodError
        }
        lockPeriod = period
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
