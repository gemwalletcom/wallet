// Copyright (c). Gem Wallet. All rights reserved.

public import enum Gemstone.GemLockPeriod
import Foundation
import Keychain
import LocalAuthentication
import Primitives

public final class LocalKeystorePassword: KeystorePassword {
    private enum Keys {
        static let password = "password"
        static let passwordAuthentication = "password_authentication"
        static let passwordAuthenticationPeriod = "password_authentication_period"
        static let lockKey = "lock_key"
    }

    private static let lock = NSLock()
    private let keychain: Keychain

    public init(keychain: Keychain = KeychainDefault()) {
        self.keychain = keychain
    }

    public func getAvailableAuthentication() -> KeystoreAuthentication {
        KeystoreAuthentication.availableAuthenticationType
    }

    public func getAuthentication() throws -> KeystoreAuthentication {
        guard let value = try keychain.get(Keys.passwordAuthentication) else {
            return .none
        }
        return KeystoreAuthentication(rawValue: value) ?? .none
    }

    public func getAuthenticationLockPeriod() throws -> GemLockPeriod? {
        guard let option = try keychain.get(Keys.passwordAuthenticationPeriod) else {
            return .none
        }
        return GemLockPeriod(keychainValue: option)
    }

    public func setAuthenticationLockPeriod(period: GemLockPeriod) throws {
        try keychain.set(period.keychainValue, key: Keys.passwordAuthenticationPeriod)
    }

    public func enableAuthentication(_ enable: Bool, context: LAContext) throws {
        try Self.lock.withLock {
            switch enable {
            case true:
                let authentication = getAvailableAuthentication()
                switch authentication {
                case .biometrics, .passcode:
                    try changeAuthentication(authentication: authentication, context: context)
                case .none:
                    throw AnyError("No authentication available")
                }
            case false:
                try changeAuthentication(authentication: .none, context: context)
                try setAuthenticationLockPeriod(period: .default)
            }
        }
    }

    public func getPassword(createIfMissing: Bool) throws -> String {
        try Self.lock.withLock {
            let context = LAContext()
            if let password = try storedPassword(context: context) {
                return password
            }
            guard createIfMissing else {
                throw KeystoreError.missingPassword
            }
            let authentication = try getAuthentication()
            let password = try generateSecret()
            try setPassword(password, authentication: authentication, context: context)
            return password
        }
    }

    public func unlock(context: LAContext) throws {
        try Self.lock.withLock {
            guard try storedLockKey(context: context) == nil else {
                return
            }
            try setLockKey(generateSecret(), authentication: getAuthentication(), context: context)
            guard try storedLockKey(context: context) != nil else {
                throw KeystoreError.missingLockKey
            }
        }
    }
}

// MARK: - Private

extension LocalKeystorePassword {
    private func generateSecret() throws -> String {
        try SecureRandom.generateKey(length: 32).hex
    }

    private func setPassword(
        _ password: String,
        authentication: KeystoreAuthentication,
        context: LAContext,
    ) throws {
        try keychain
            .accessibility(.whenUnlockedThisDeviceOnly, authenticationPolicy: authentication.policy)
            .authenticationContext(context)
            .set(password, key: Keys.password)
    }

    private func storedPassword(context: LAContext) throws -> String? {
        guard let password = try keychain.authenticationContext(context).get(Keys.password), password.isNotEmpty else {
            return nil
        }
        return password
    }

    private func storedLockKey(context: LAContext) throws -> String? {
        try keychain.authenticationContext(context).get(Keys.lockKey)
    }

    private func setLockKey(_ lockKey: String, authentication: KeystoreAuthentication, context: LAContext) throws {
        try keychain.remove(Keys.lockKey)
        try keychain
            .accessibility(.whenUnlockedThisDeviceOnly, authenticationPolicy: authentication.policy)
            .authenticationContext(context)
            .set(lockKey, key: Keys.lockKey)
    }

    private func changeAuthentication(authentication: KeystoreAuthentication, context: LAContext) throws {
        let password = try storedPassword(context: context)
        let lockKey = try storedLockKey(context: context) ?? generateSecret()
        try keychain.set(authentication.rawValue, key: Keys.passwordAuthentication)
        if let password {
            try setPassword(password, authentication: authentication, context: context)
        }
        try setLockKey(lockKey, authentication: authentication, context: context)
    }
}

// MARK: - Models extensions

extension LAContext {
    func canEvaluatePolicyThrowing(policy: LAPolicy) throws {
        var error: NSError?
        canEvaluatePolicy(policy, error: &error)
        if let error {
            throw error
        }
    }
}

private extension GemLockPeriod {
    init?(keychainValue: String) {
        switch keychainValue {
        case "immediate": self = .immediate
        case "oneMinute": self = .oneMinute
        case "fiveMinutes": self = .fiveMinutes
        case "fifteenMinutes": self = .fifteenMinutes
        case "oneHour": self = .oneHour
        case "sixHours": self = .sixHours
        default: return nil
        }
    }

    var keychainValue: String {
        switch self {
        case .immediate: "immediate"
        case .oneMinute: "oneMinute"
        case .fiveMinutes: "fiveMinutes"
        case .fifteenMinutes: "fifteenMinutes"
        case .oneHour: "oneHour"
        case .sixHours: "sixHours"
        }
    }
}
