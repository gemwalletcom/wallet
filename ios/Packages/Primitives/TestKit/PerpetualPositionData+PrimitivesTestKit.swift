// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension PerpetualPositionData {
    static func mock(perpetual: Perpetual = .mock(), position: PerpetualPosition = .mock()) -> PerpetualPositionData {
        PerpetualPositionData(
            perpetual: perpetual,
            asset: .mock(),
            position: position,
        )
    }
}
