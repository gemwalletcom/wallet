// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.PerpetualReduceData {
    func map() throws -> Primitives.PerpetualReduceData {
        try Primitives.PerpetualReduceData(
            data: data.map(),
            positionDirection: positionDirection.map(),
        )
    }
}
