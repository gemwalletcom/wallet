// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct LightningPendingSend: Equatable, Sendable {
    public let transferId: String
    public let invoice: String
    public let amountSats: UInt64?
    public let expiresAt: UInt64

    public init(transferId: String, invoice: String, amountSats: UInt64?, expiresAt: UInt64) {
        self.transferId = transferId
        self.invoice = invoice
        self.amountSats = amountSats
        self.expiresAt = expiresAt
    }
}
