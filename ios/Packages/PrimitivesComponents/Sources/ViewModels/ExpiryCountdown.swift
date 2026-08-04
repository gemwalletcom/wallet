// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

@Observable
@MainActor
public final class ExpiryCountdown {
    public private(set) var isExpired: Bool = false

    private let expiresAt: Date?

    public init(expiresAt: Date?) {
        self.expiresAt = expiresAt
    }

    public func start() async {
        guard let expiresAt else {
            return
        }
        await expiresAt.sleepUntil()
        isExpired = true
    }
}
