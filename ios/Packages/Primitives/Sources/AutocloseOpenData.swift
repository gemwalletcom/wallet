// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct AutocloseOpenData: Codable, Equatable, Hashable, Sendable {
    public let assetId: AssetId
    public let symbol: String
    public let direction: PerpetualDirection
    public let marketPrice: Double
    public let leverage: UInt8
    public let size: Double
    public let assetDecimals: Int32
    public let takeProfit: String?
    public let stopLoss: String?

    public init(assetId: AssetId, symbol: String, direction: PerpetualDirection, marketPrice: Double, leverage: UInt8, size: Double, assetDecimals: Int32, takeProfit: String?, stopLoss: String?) {
        self.assetId = assetId
        self.symbol = symbol
        self.direction = direction
        self.marketPrice = marketPrice
        self.leverage = leverage
        self.size = size
        self.assetDecimals = assetDecimals
        self.takeProfit = takeProfit
        self.stopLoss = stopLoss
    }
}
