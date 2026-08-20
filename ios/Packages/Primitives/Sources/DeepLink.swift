// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum DeepLink: Equatable, Sendable {
    case asset(AssetId)
    case perpetuals
    case rewards(code: String?)
    case receive(AssetId?)
    case buy(AssetId, amount: Int?)
    case sell(AssetId, amount: Int?)
}
