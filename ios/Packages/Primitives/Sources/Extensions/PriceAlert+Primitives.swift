// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension PriceAlert {
    static func `default`(for assetId: AssetId, currency: Currency) -> PriceAlert {
        PriceAlert(
            assetId: assetId,
            currency: currency,
            price: .none,
            pricePercentChange: .none,
            priceDirection: .none,
            lastNotifiedAt: .none,
        )
    }
}
