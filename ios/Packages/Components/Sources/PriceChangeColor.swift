// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public enum PriceChangeColor {
    public static func color(for value: Double) -> Color {
        switch value {
        case _ where value > 0:
            Colors.green
        case _ where value < 0:
            Colors.red
        default:
            Colors.gray
        }
    }
}
