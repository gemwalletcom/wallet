// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct DeviceToken: Codable, Equatable, Sendable {
    public let token: String
    public let expiresAt: UInt64

    public init(token: String, expiresAt: UInt64) {
        self.token = token
        self.expiresAt = expiresAt
    }
}
