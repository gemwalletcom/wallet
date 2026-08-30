// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

enum SetPriceAlertType {
    case price
    case percentage

    var notificationType: PriceAlertNotificationType {
        switch self {
        case .price: .price
        case .percentage: .pricePercentChange
        }
    }
}
