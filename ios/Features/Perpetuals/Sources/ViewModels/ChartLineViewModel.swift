// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import struct Gemstone.GemPerpetualChartLine
import Localization
import Style
import SwiftUI

struct ChartLineViewModel: Identifiable {
    let line: GemPerpetualChartLine
    let formatter: NumericFormatter

    var id: String {
        "\(line.kind)_\(line.price)"
    }

    var price: Double {
        line.price
    }

    var label: String {
        let typeLabel: String = switch line.kind {
        case .takeProfit: Localized.Perpetual.takeProfit
        case .stopLoss: Localized.Perpetual.stopLoss
        case .entry: Localized.Charts.entry
        case .liquidation: Localized.Perpetual.liquidation
        }
        let priceText = formatter.string(line.price)
        return "\(typeLabel) | \(priceText)"
    }

    var color: Color {
        switch line.kind {
        case .takeProfit: Colors.green
        case .stopLoss: Colors.orange
        case .entry: Colors.gray
        case .liquidation: Colors.red
        }
    }

    var lineStyle: StrokeStyle {
        StrokeStyle(lineWidth: 1, dash: [4, 3])
    }
}
