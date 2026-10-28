// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemPriceAlertItem
import enum Gemstone.GemPriceAlertSectionKind

extension GemPriceAlertItem: @retroactive Identifiable {}

extension GemPriceAlertSectionKind {
    var id: String {
        switch self {
        case .auto: "auto"
        case let .asset(assetId, _): assetId.identifier
        }
    }
}
