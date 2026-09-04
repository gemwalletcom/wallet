// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct AutocloseSelection: Sendable {
    public let takeProfit: String?
    public let stopLoss: String?

    public init(takeProfit: String?, stopLoss: String?) {
        self.takeProfit = takeProfit
        self.stopLoss = stopLoss
    }
}
