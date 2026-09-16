// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import LocalAuthentication

@MainActor
public final class BiometryAuthenticationService: BiometryAuthenticatable {
    private let keystorePassword: KeystorePassword
    private let reason: String

    public private(set) var isAuthenticating = false

    public nonisolated init(keystorePassword: KeystorePassword, reason: String) {
        self.keystorePassword = keystorePassword
        self.reason = reason
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
            return true
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

    public func enableAuthentication(_ enable: Bool, context: LAContext) async throws {
        try await authenticate(context: context)
        try keystorePassword.enableAuthentication(enable, context: context)
    }

    public func authenticate(context: LAContext) async throws {
        isAuthenticating = true
        defer { isAuthenticating = false }
        do {
            try await context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: reason)
        } catch let error as NSError {
            throw BiometryAuthenticationError(error: error)
        }
    }
}
