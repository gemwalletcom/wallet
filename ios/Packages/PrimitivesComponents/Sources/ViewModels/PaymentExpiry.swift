// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

@Observable
@MainActor
public final class PaymentExpiry {
    public private(set) var isExpired: Bool = false

    private let expiresAt: Date?

    public init(payment: PaymentData?) {
        expiresAt = payment?.expiresAt
    }

    public func start() async {
        guard let expiresAt else {
            return
        }
        await expiresAt.sleepUntil()
        isExpired = true
    }
}
