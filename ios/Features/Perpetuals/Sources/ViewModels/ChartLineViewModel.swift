// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemPerpetualChartLine
import Style
import SwiftUI

struct ChartLineViewModel: Identifiable {
    let line: GemPerpetualChartLine

    var id: String {
        "\(line.kind)_\(line.price.value)"
    }

    var price: Double {
        line.price.value
    }

    var label: String {
        "\(line.kind.title) | \(line.price.text())"
    }

    var color: Color {
        line.kind.color
    }

    var lineStyle: StrokeStyle {
        StrokeStyle(lineWidth: 1, dash: [4, 3])
    }
}
