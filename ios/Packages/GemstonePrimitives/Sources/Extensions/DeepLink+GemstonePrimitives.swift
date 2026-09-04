// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.DeepLink {
    func map() -> Gemstone.Deeplink {
        switch self {
        case let .asset(assetId): .asset(assetId: assetId.identifier)
        case .perpetuals: .perpetuals
        case let .rewards(code): .rewards(code: code)
        case let .receive(assetId): .receive(assetId: assetId.identifier)
        case let .buy(assetId, amount): .buy(assetId: assetId.identifier, amount: amount.map(\.asInt32))
        case let .sell(assetId, amount): .sell(assetId: assetId.identifier, amount: amount.map(\.asInt32))
        case let .swap(assetId): .swap(assetId: assetId.identifier)
        }
    }
}

public extension Gemstone.Deeplink {
    func map() throws -> Primitives.DeepLink {
        switch self {
        case let .asset(assetId): try .asset(AssetId(id: assetId))
        case .perpetuals: .perpetuals
        case let .rewards(code): .rewards(code: code)
        case let .receive(assetId): try .receive(AssetId(id: assetId))
        case let .buy(assetId, amount): try .buy(AssetId(id: assetId), amount: amount.map(\.asInt))
        case let .sell(assetId, amount): try .sell(AssetId(id: assetId), amount: amount.map(\.asInt))
        case let .swap(assetId): try .swap(AssetId(id: assetId))
        }
    }
}
