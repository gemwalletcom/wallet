// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

enum PerpetualError: Equatable {
    case invalidAutoclose(type: TpslType, direction: PerpetualDirection)
}

extension PerpetualError: LocalizedError {
    var errorDescription: String? {
        switch self {
        case let .invalidAutoclose(type, direction):
            switch (type, direction) {
            case (.takeProfit, .long), (.stopLoss, .short): Localized.Errors.Perpetual.triggerPriceHigher
            case (.takeProfit, .short), (.stopLoss, .long): Localized.Errors.Perpetual.triggerPriceLower
            }
        }
    }
}
