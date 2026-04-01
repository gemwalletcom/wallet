// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.GemPerpetualMarginType {
    func map() -> Primitives.PerpetualMarginType {
        switch self {
        case .cross: .cross
        case .isolated: .isolated
        }
    }
}

public extension Primitives.PerpetualMarginType {
    func map() -> Gemstone.GemPerpetualMarginType {
        switch self {
        case .cross: .cross
        case .isolated: .isolated
        }
    }
}
