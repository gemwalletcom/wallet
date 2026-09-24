// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAssetFilter
import Store

public extension GemAssetFilter {
    func map() -> AssetsRequestFilter {
        switch self {
        case .enabled: .enabled
        case .buyable: .buyable
        case .sellable: .sellable
        case .swappable: .swappable
        case .hasBalance: .hasBalance
        case .hasAvailableBalance: .hasAvailableBalance
        case let .chainsOrAssetIds(chains, assetIds): .chainsOrAssets(chains, assetIds)
        case let .chains(chains): .chains(chains)
        }
    }
}
