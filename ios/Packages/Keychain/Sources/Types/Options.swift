// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct Options: @unchecked Sendable {
    var service: String = ""
    var accessibility: Accessibility = .afterFirstUnlock
    var authenticationPolicy: AuthenticationPolicy?
    var authenticationContext: AnyObject?
}

extension Options {
    func query() -> [String: Any] {
        var query = [String: Any]()

        query[Class] = String(kSecClassGenericPassword)
        query[AttributeSynchronizable] = SynchronizableAny
        query[AttributeService] = service

        if authenticationContext != nil {
            query[UseAuthenticationContext] = authenticationContext
        }

        return query
    }

    func attributes(key: String?, value: Data) -> ([String: Any], Error?) {
        var attributes: [String: Any]

        if key != nil {
            attributes = query()
            attributes[AttributeAccount] = key
        } else {
            attributes = [String: Any]()
        }

        attributes[ValueData] = value

        if let policy = authenticationPolicy {
            var error: Unmanaged<CFError>?
            guard
                let accessControl = SecAccessControlCreateWithFlags(kCFAllocatorDefault, accessibility.rawValue as CFTypeRef, SecAccessControlCreateFlags(rawValue: CFOptionFlags(policy.rawValue)), &error)
            else {
                if let error = error?.takeUnretainedValue() {
                    return (attributes, error.error)
                }

                return (attributes, KeychainError.unexpectedError)
            }
            attributes[AttributeAccessControl] = accessControl
        } else {
            attributes[AttributeAccessible] = accessibility.rawValue
        }

        attributes[AttributeSynchronizable] = kCFBooleanFalse

        return (attributes, nil)
    }
}
