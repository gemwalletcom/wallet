// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct PaymentQuotesRequest: Identifiable, Sendable {
    public let id: String
    public let quotes: PaymentQuotes
    public let wallet: Wallet
    public let assetsData: [AssetData]

    public init(id: String, quotes: PaymentQuotes, wallet: Wallet, assetsData: [AssetData]) {
        self.id = id
        self.quotes = quotes
        self.wallet = wallet
        self.assetsData = assetsData
    }

    public func assetData(for quote: PaymentQuote) -> AssetData? {
        assetsData.first { $0.asset.id == quote.amount.assetId }
    }
}
