// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemLockPeriod
import GemstoneServices
import LocalAuthentication
import Primitives

public final class MockKeystorePassword: KeystorePassword, @unchecked Sendable {
    public private(set) var getPasswordCallsCount = 0
    public var getAuthenticationError: (any Error)?

    private let passwordLock = NSLock()
    private var memoryPassword: String
    private var isAuthenticationEnabled: Bool
    private var lockPeriod: GemLockPeriod?
    private var availableAuthentication: KeystoreAuthentication

    public init(
        memoryPassword: String = "",
        isAuthenticationEnabled: Bool = false,
        lockPeriod: GemLockPeriod? = .default,
        availableAuthentication: KeystoreAuthentication = .none,
    ) {
        self.memoryPassword = memoryPassword
        self.isAuthenticationEnabled = isAuthenticationEnabled
        self.availableAuthentication = availableAuthentication
        self.lockPeriod = lockPeriod
    }

    public func getPassword(createIfMissing: Bool) throws -> String {
        try passwordLock.withLock {
            getPasswordCallsCount += 1
            if memoryPassword.isNotEmpty {
                return memoryPassword
            }
            guard createIfMissing else {
                throw KeystoreError.missingPassword
            }
            _ = try getAuthentication()
            memoryPassword = try SecureRandom.generateKey(length: 32).hex
            return memoryPassword
        }
    }

    public func getAuthentication() throws -> KeystoreAuthentication {
        if let getAuthenticationError {
            throw getAuthenticationError
        }
        return availableAuthentication
    }

    public func getAvailableAuthentication() -> KeystoreAuthentication {
        availableAuthentication
    }

    public func getAuthenticationLockPeriod() throws -> GemLockPeriod? {
        lockPeriod
    }

    public func setAuthenticationLockPeriod(period: GemLockPeriod) throws {
        lockPeriod = period
    }

    public func enableAuthentication(_ enable: Bool, context _: LAContext) throws {
        isAuthenticationEnabled = enable
    }

    public func unlock(context _: LAContext) throws {}
}
