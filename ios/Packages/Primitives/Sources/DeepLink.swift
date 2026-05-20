// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum DeepLink: Equatable, Sendable {
    case asset(AssetId)
    case swap(AssetId, AssetId?)
    case perpetuals
    case rewards(code: String?)
    case gift(code: String?)
    case buy(AssetId, amount: Int?)
    case sell(AssetId, amount: Int?)
    case setPriceAlert(AssetId, price: Double?)
}
