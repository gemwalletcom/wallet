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
        static let passwordAuthenticationPrivacyLock = "password_authentication_privacy_lock"
    }

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

    public func getPrivacyLockStatus() throws -> PrivacyLockStatus? {
        guard let value = try keychain.get(Keys.passwordAuthenticationPrivacyLock) else {
            return .none
        }
        return PrivacyLockStatus(rawValue: value) ?? .none
    }

    public func setPrivacyLockStatus(_ status: PrivacyLockStatus) throws {
        try keychain.set(status.rawValue, key: Keys.passwordAuthenticationPrivacyLock)
    }

    public func setAuthenticationLockPeriod(period: GemLockPeriod) throws {
        try keychain.set(period.keychainValue, key: Keys.passwordAuthenticationPeriod)
    }

    public func enableAuthentication(_ enable: Bool, context: LAContext) throws {
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
            try setPrivacyLockStatus(.disabled)
            try setAuthenticationLockPeriod(period: .default)
        }
    }

    public func getPassword() throws -> String {
        try getPassword(context: LAContext())
    }

    public func getPassword(context: LAContext) throws -> String {
        try keychain
            .authenticationContext(context)
            .get(Keys.password) ?? ""
    }

    public func setPassword(_ password: String, authentication: KeystoreAuthentication) throws {
        try setPassword(password, authentication: authentication, context: LAContext())
    }

    public func remove() throws {
        try keychain.remove(Keys.password)
        try keychain.remove(Keys.passwordAuthentication)
        try keychain.remove(Keys.passwordAuthenticationPeriod)
        try keychain.remove(Keys.passwordAuthenticationPrivacyLock)
    }
}

// MARK: - Private

extension LocalKeystorePassword {
    private func setPassword(
        _ password: String,
        authentication: KeystoreAuthentication,
        context: LAContext,
    ) throws {
        guard password.isNotEmpty else {
            throw KeystoreError.emptyPassword
        }
        try keychain
            .set(authentication.rawValue, key: Keys.passwordAuthentication)

        try keychain
            .accessibility(.whenUnlockedThisDeviceOnly, authenticationPolicy: authentication.policy)
            .authenticationContext(context)
            .set(password, key: Keys.password)
    }

    private func changeAuthentication(authentication: KeystoreAuthentication, context: LAContext) throws {
        let password = try getPassword(context: context)
        guard password.isNotEmpty else {
            return try keychain.set(authentication.rawValue, key: Keys.passwordAuthentication)
        }
        try setPassword(password, authentication: authentication, context: context)
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
