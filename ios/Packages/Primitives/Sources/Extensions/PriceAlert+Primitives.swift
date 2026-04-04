// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension PriceAlert: Identifiable {
    public var id: String {
        if price == nil, pricePercentChange == nil, priceDirection == nil {
            return assetId.identifier
        }
        return [
            assetId.identifier,
            currency,
            price.map { Self.formatValue($0) },
            pricePercentChange.map { Self.formatValue($0) },
            priceDirection?.rawValue,
        ]
        .compactMap(\.self)
        .joined(separator: "_")
    }
}

public extension PriceAlert {
    static func `default`(for assetId: AssetId, currency: String) -> PriceAlert {
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

    private static func formatValue(_ value: Double) -> String {
        let str = String(value)
        return str.hasSuffix(".0") ? String(str.dropLast(2)) : str
    }
}
