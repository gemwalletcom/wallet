// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPriceAlertSectionKind

extension GemPriceAlertSectionKind {
    var id: String {
        switch self {
        case .auto: "auto"
        case let .asset(assetId, _): assetId.identifier
        }
    }
}
