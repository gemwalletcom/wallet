// Copyright (c). Gem Wallet. All rights reserved.

public import enum Gemstone.GemKeystoreAuthentication
public import enum Gemstone.GemLockPeriod
import protocol Gemstone.GemSecurityServiceProtocol
import LocalAuthentication
import Primitives

public struct BiometryAuthenticationService: BiometryAuthenticatable {
    private let keystorePassword: KeystorePassword
    private let securityService: any GemSecurityServiceProtocol

    public init(
        keystorePassword: KeystorePassword,
        securityService: any GemSecurityServiceProtocol,
    ) {
        self.keystorePassword = keystorePassword
        self.securityService = securityService
    }

    public func shouldRelock(elapsedMilliseconds: Int64) -> Bool {
        securityService.shouldRelock(
            elapsedMilliseconds: elapsedMilliseconds,
            lockIntervalMinutes: lockPeriod.minutes(),
            authRequired: requiresAuthentication,
        )
    }

    public var requiresAuthentication: Bool {
        do {
            return try keystorePassword.getAuthentication() != .none
        } catch {
            return true
        }
    }

    public var lockPeriod: GemLockPeriod {
        do {
            return try keystorePassword.getAuthenticationLockPeriod() ?? .default
        } catch {
            return .default
        }
    }

    public func update(period: GemLockPeriod) throws {
        try keystorePassword.setAuthenticationLockPeriod(period: period)
    }

    public var availableAuthentication: GemKeystoreAuthentication {
        keystorePassword.getAvailableAuthentication().toGem()
    }

    public var isPasscodeSet: Bool {
        var error: NSError?
        LAContext().canEvaluatePolicy(.deviceOwnerAuthentication, error: &error)
        return error?.code != LAError.passcodeNotSet.rawValue
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
        guard requiresAuthentication else { return }
        do {
            try keystorePassword.unlock(context: context)
        } catch where error.isAuthenticationCancelled {
            throw BiometryAuthenticationError.cancelledByUser
        } catch {
            debugLog("lock key unlock failed: \(error)")
            throw BiometryAuthenticationError.authenticationFailed
        }
    }
}
