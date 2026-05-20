// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.DeepLink {
    var url: URL {
        Gemstone.deeplinkBuildUrl(deeplink: map()).asURL!
    }

    var gemUrl: URL {
        Gemstone.deeplinkBuildGemUrl(deeplink: map()).asURL!
    }

    func map() -> Gemstone.Deeplink {
        switch self {
        case let .asset(assetId): .asset(assetId: assetId.identifier)
        case let .swap(fromAssetId, toAssetId): .swap(fromAssetId: fromAssetId.identifier, toAssetId: toAssetId?.identifier)
        case .perpetuals: .perpetuals
        case let .rewards(code): .rewards(code: code)
        case let .gift(code): .gift(code: code)
        case let .buy(assetId, amount): .buy(assetId: assetId.identifier, amount: amount.map { Int32($0) })
        case let .sell(assetId, amount): .sell(assetId: assetId.identifier, amount: amount.map { Int32($0) })
        case let .setPriceAlert(assetId, price): .setPriceAlert(assetId: assetId.identifier, price: price)
        }
    }
}
