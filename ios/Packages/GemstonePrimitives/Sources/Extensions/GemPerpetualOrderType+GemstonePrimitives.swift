// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.GemPerpetualOrderType {
    func map() -> Primitives.PerpetualOrderType {
        switch self {
        case .market: .market
        case .limit: .limit
        }
    }
}

public extension Primitives.PerpetualOrderType {
    func map() -> Gemstone.GemPerpetualOrderType {
        switch self {
        case .market: .market
        case .limit: .limit
        }
    }
}
