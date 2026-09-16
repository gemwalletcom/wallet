// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSwapProgressMarker
import Style
import SwiftUI

extension GemSwapProgressMarker {
    var image: Image? {
        switch self {
        case .check: Images.System.checkmark
        case .dots: Images.System.ellipsis
        case .cross: Images.System.xmark
        case .swap: Images.System.arrowSwap
        case .spinner: nil
        }
    }
}
