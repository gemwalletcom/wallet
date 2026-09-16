// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension PerpetualPositionData {
    static func mock(position: PerpetualPosition = .mock()) -> PerpetualPositionData {
        PerpetualPositionData(
            perpetual: .mock(),
            asset: .mock(),
            position: position,
        )
    }
}
