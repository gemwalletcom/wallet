// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PaymentActionResults: Sendable {
    public let results: [String]
    public let transactionHash: String?
}
