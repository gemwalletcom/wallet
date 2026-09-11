// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PriceAlertData: Codable, Equatable, Hashable, Sendable {
    public let asset: Asset
    public let price: Price?
    public let priceAlert: PriceAlert

    public init(asset: Asset, price: Price?, priceAlert: PriceAlert) {
        self.asset = asset
        self.price = price
        self.priceAlert = priceAlert
    }
}
