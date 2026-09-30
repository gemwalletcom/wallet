// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct AuthenticationPolicy: OptionSet, Sendable {
    public static let biometryAny = AuthenticationPolicy(rawValue: 1 << 1)
    public static let devicePasscode = AuthenticationPolicy(rawValue: 1 << 4)
    public static let or = AuthenticationPolicy(rawValue: 1 << 14)

    public let rawValue: UInt

    public init(rawValue: UInt) {
        self.rawValue = rawValue
    }
}
