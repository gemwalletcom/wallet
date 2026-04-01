// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPerpetualPositionsSummary {
    func map() throws -> PerpetualPositionsSummary {
        try PerpetualPositionsSummary(
            positions: positions.map { try $0.map() },
            balance: balance.map(),
        )
    }
}
