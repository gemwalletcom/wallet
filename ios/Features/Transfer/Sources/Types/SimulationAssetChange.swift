// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import GemstonePrimitives
import Primitives

struct SimulationAssetChange: Equatable {
    let asset: Asset
    let value: BigInt
}

extension AssetId {
    func unresolvedSimulationAsset() -> Asset {
        let identifier = tokenId?.truncate(first: 6, last: 6) ?? chain.rawValue
        let type: AssetType = tokenId == nil ? .native : (assetType ?? .token)

        return Asset(
            id: self,
            name: identifier,
            symbol: identifier,
            decimals: 0,
            type: type,
        )
    }
}
