// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

/// The three-way tone the app reads from Core's `valueTone`, restated here because the widget
/// extension cannot link Gemstone. A change of nothing is neutral, not a gain. Keep the two in
/// step: U17 removes this copy once the widget can reach Core.
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
