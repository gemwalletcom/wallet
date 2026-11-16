// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension PriceData: Identifiable {
    public var id: String {
        asset.id.identifier
    }
}

public extension PriceData {
    static func with(asset: Asset) -> PriceData {
        PriceData(
            asset: asset,
            price: nil,
            priceAlerts: [],
            market: nil,
            links: [],
            properties: AssetProperties(
                isEnabled: false,
                isBuyable: false,
                isSellable: false,
                isSwapable: false,
                isStakeable: false,
                stakingApr: nil,
                isEarnable: false,
                earnApr: nil,
                hasImage: false,
                hasCandles: false,
            ),
        )
    }
}
