// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import LocalAuthentication

internal import Security

// Adapted from: https://github.com/kishikawakatsumi/KeychainAccess/blob/master/Lib/KeychainAccess/Keychain.swift

public final class KeychainDefault: Keychain {
    fileprivate let options: Options

    // MARK: - Constructors

    public convenience init() {
        var options = Options()
        if let bundleIdentifier = Bundle.main.bundleIdentifier {
            options.service = bundleIdentifier
        }
        self.init(options)
    }

    fileprivate init(_ opts: Options) {
        options = opts
    }

    // MARK: - Public (Set Options) methods

    public func accessibility(_ accessibility: Accessibility, authenticationPolicy: AuthenticationPolicy) -> Keychain {
        var options = options
        options.accessibility = accessibility
        options.authenticationPolicy = authenticationPolicy
        return KeychainDefault(options)
    }

    public func authenticationContext(_ authenticationContext: LAContext) -> Keychain {
        var options = options
        options.authenticationContext = authenticationContext
        return KeychainDefault(options)
    }

    // MARK: - Public (get) methods

    public func get(_ key: String) throws -> String? {
        guard let data = try getData(key) else {
            return nil
        }
        guard let string = String(data: data, encoding: .utf8) else {
            throw KeychainError.conversionError
        }
        return string
    }

    public func getData(_ key: String) throws -> Data? {
        var query = options.query()

        query[MatchLimit] = MatchLimitOne
        query[ReturnData] = kCFBooleanTrue

        query[AttributeAccount] = key

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        switch status {
        case errSecSuccess:
            guard let data = result as? Data else {
                throw KeychainError.unexpectedError
            }
            return data
        case errSecItemNotFound:
            return nil
        default:
            throw KeychainError(status: status)
        }
    }

    // MARK: - Public (set) methods

    public func set(_ value: String, key: String) throws {
        guard let data = value.data(using: .utf8, allowLossyConversion: false) else {
            throw KeychainError.conversionError
        }
        try set(data, key: key)
    }

    public func set(_ value: Data, key: String) throws {
        var query = options.query()
        query[AttributeAccount] = key

        var status = SecItemCopyMatching(query as CFDictionary, nil)
        switch status {
        case errSecSuccess, errSecInteractionNotAllowed:
            var query = options.query()
            query[AttributeAccount] = key

            let (attributes, error) = options.attributes(key: nil, value: value)
            if let error {
                throw error
            }

            status = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
            if status != errSecSuccess {
                throw KeychainError(status: status)
            }
        case errSecItemNotFound:
            let (attributes, error) = options.attributes(key: key, value: value)
            if let error {
                throw error
            }

            status = SecItemAdd(attributes as CFDictionary, nil)
            if status != errSecSuccess {
                throw KeychainError(status: status)
            }
        default:
            throw KeychainError(status: status)
        }
    }

    // MARK: - Public (remove) methods

    public func remove(_ key: String) throws {
        var query = options.query()
        query[AttributeAccount] = key

        let status = SecItemDelete(query as CFDictionary)
        if status != errSecSuccess, status != errSecItemNotFound {
            throw KeychainError(status: status)
        }
    }
}
