// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.TpslType {
    func map() -> Gemstone.TpslType {
        switch self {
        case .takeProfit: .takeProfit
        case .stopLoss: .stopLoss
        }
    }
}
