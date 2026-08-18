// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Asset {
    init(_ chain: Chain) {
        let asset = chain.asset
        self.init(
            id: chain.assetId,
            name: asset.name,
            symbol: asset.symbol,
            decimals: asset.decimals,
            type: asset.type,
        )
    }

    var feeAsset: Asset {
        switch id.chain {
        case .hyperCore:
            switch map().assetType {
            case .native: Asset.hypercoreSpotUSDC()
            case .perpetual: Asset.hypercoreUSDC()
            default: Asset.hypercoreSpotUSDC()
            }
        default:
            switch id.type {
            case .native: self
            case .token: id.chain.asset
            }
        }
    }

    var defaultBasic: AssetBasic {
        AssetBasic(
            asset: self,
            properties: .defaultValue(assetId: id),
            score: .defaultValue(assetId: id),
            price: nil,
        )
    }
}
