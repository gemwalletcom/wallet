// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI
import WidgetKit

@Observable
@MainActor
final class PriceWidgetViewModel {
    let entry: PriceWidgetEntry
    let widgetFamily: WidgetFamily
    init(
        entry: PriceWidgetEntry,
        widgetFamily: WidgetFamily,
    ) {
        self.entry = entry
        self.widgetFamily = widgetFamily
    }

    var prices: [CoinPrice] {
        switch widgetFamily {
        case .systemSmall:
            Array(entry.coinPrices.prefix(1))
        case .systemMedium:
            Array(entry.coinPrices.prefix(3))
        case .systemLarge:
            entry.coinPrices
        default:
            entry.coinPrices
        }
    }

    var emptyMessage: String {
        switch widgetFamily {
        case .systemSmall:
            "No data"
        default:
            "No price data available"
        }
    }
}
