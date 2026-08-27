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

    var type: PriceAlertNotificationType {
        switch (priceDirection, price, pricePercentChange) {
        case (nil, nil, nil): .auto
        case (.some, .some, nil): .price
        case (.some, nil, .some): .pricePercentChange
        default: .auto
        }
    }

    var shouldDisplay: Bool {
        switch type {
        case .auto: true
        case .price, .pricePercentChange: lastNotifiedAt == nil
        }
    }
}
