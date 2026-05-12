// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

public extension PerpetualMarginType {
    var title: String {
        switch self {
        case .cross: Localized.Perpetual.Margin.cross
        case .isolated: Localized.Perpetual.Margin.isolated
        }
    }
}
