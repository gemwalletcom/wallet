// Copyright (c). Gem Wallet. All rights reserved.

public import enum Gemstone.GemLockPeriod
import Foundation
import LocalAuthentication
import Primitives

public protocol BiometryAuthenticatable: Sendable {
    var requiresAuthentication: Bool { get }
    var availableAuthentication: KeystoreAuthentication { get }
    var isPasscodeSet: Bool { get }
    var lockPeriod: GemLockPeriod { get }

    @MainActor
    func authenticate(context: LAContext, reason: String) async throws
    @MainActor
    func enableAuthentication(_ enable: Bool, context: LAContext, reason: String) async throws
    func update(period: GemLockPeriod) throws
    func shouldRelock(elapsedMilliseconds: Int64) -> Bool
}

public extension BiometryAuthenticatable {
    @MainActor
    func authenticate(reason: String) async throws {
        try await authenticate(context: LAContext(), reason: reason)
    }

    @MainActor
    func authenticateIfRequired(reason: String) async throws -> Bool {
        guard requiresAuthentication else { return true }
        do {
            try await authenticate(reason: reason)
            return true
        } catch is BiometryAuthenticationError {
            return false
        }
    }

    @MainActor
    func enableAuthentication(_ enable: Bool, reason: String) async throws {
        try await enableAuthentication(enable, context: LAContext(), reason: reason)
    }
}
