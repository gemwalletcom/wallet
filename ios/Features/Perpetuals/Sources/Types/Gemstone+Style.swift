// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualChartLineKind
import Style
import SwiftUI

extension GemPerpetualChartLineKind {
    var color: Color {
        switch self {
        case .takeProfit: Colors.green
        case .stopLoss: Colors.orange
        case .entry: Colors.gray
        case .liquidation: Colors.red
        }
    }
}
