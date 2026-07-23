// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization

enum PerpetualError: Equatable {
    case triggerPriceMustBeHigher
    case triggerPriceMustBeLower
}

extension PerpetualError: LocalizedError {
    var errorDescription: String? {
        switch self {
        case .triggerPriceMustBeHigher: Localized.Errors.Perpetual.triggerPriceHigher
        case .triggerPriceMustBeLower: Localized.Errors.Perpetual.triggerPriceLower
        }
    }
}
