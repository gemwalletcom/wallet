// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAssetAction
import Primitives

public extension SelectedAssetType {
    var action: GemAssetAction? {
        switch self {
        case .send: .send
        case .receive: .receive
        case .buy: .buy
        case .sell: .sell
        case .stake, .earn, .swap: .none
        }
    }
}
