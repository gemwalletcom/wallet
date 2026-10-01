// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

enum WidgetValueTone {
    case positive
    case negative
    case neutral

    init(change: Double) {
        self = if change > 0 {
            .positive
        } else if change < 0 {
            .negative
        } else {
            .neutral
        }
    }

    var color: Color {
        switch self {
        case .positive: Colors.green
        case .negative: Colors.red
        case .neutral: Colors.gray
        }
    }
}
