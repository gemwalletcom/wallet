// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemSecurityService
import Primitives
import LocalAuthentication

public struct BiometryAuthenticationService: BiometryAuthenticatable {
    private let keystorePassword: KeystorePassword
    private let securityService: GemSecurityService

    public init(
        keystorePassword: KeystorePassword = LocalKeystorePassword(),
        securityService: GemSecurityService = GemSecurityService(),
    ) {
        self.keystorePassword = keystorePassword
        self.securityService = securityService
    }

    public func shouldRelock(elapsedMilliseconds: Int64) -> Bool {
        securityService.shouldRelock(
            elapsedMilliseconds: elapsedMilliseconds,
            lockIntervalMinutes: securityService.lockPeriodMinutes(period: lockPeriod.gemLockPeriod),
            authRequired: requiresAuthentication,
            hasPendingRequest: false,
        )
    }

    public var requiresAuthentication: Bool {
        do {
            return try keystorePassword.getAuthentication() != .none
        } catch {
            return true
        }
    }

    public var isPrivacyLockEnabled: Bool {
        do {
            return try keystorePassword.getPrivacyLockStatus() == .enabled
        } catch {
            return false
        }
    }

    public func togglePrivacyLock(enabled: Bool) throws {
        let status = PrivacyLockStatus(enabled: enabled)
        try keystorePassword.setPrivacyLockStatus(status)
    }

    public var lockPeriod: LockPeriod {
        do {
            return try keystorePassword.getAuthenticationLockPeriod() ?? .default
        } catch {
            return .default
        }
    }

    public func update(period: LockPeriod) throws {
        try keystorePassword.setAuthenticationLockPeriod(period: period)
    }

    public var availableAuthentication: KeystoreAuthentication {
        keystorePassword.getAvailableAuthentication()
    }

    @MainActor
    public func enableAuthentication(_ enable: Bool, context: LAContext, reason: String) async throws {
        try await authenticate(context: context, reason: reason)
        try keystorePassword.enableAuthentication(enable, context: context)
    }

    @MainActor
    public func authenticate(context: LAContext, reason: String) async throws {
        do {
            try await context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: reason)
        } catch let error as NSError {
            throw BiometryAuthenticationError(error: error)
        }
    }
}
