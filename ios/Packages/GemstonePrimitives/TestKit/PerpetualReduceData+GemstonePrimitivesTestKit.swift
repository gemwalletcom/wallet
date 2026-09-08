// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.PerpetualConfirmData
import enum Gemstone.PerpetualDirection
import struct Gemstone.PerpetualReduceData

public extension PerpetualReduceData {
    static func mock(
        data: PerpetualConfirmData = .mock(),
        positionDirection: PerpetualDirection = .long,
    ) -> PerpetualReduceData {
        PerpetualReduceData(
            data: data,
            positionDirection: positionDirection,
        )
    }
}
