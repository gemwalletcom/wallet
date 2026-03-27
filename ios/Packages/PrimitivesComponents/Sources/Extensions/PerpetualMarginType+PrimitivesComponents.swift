// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension PerpetualMarginType {
    var title: String {
        switch self {
        case .cross: "Cross"
        case .isolated: "Isolated"
        }
    }
}
