// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemLeverageSelection
import enum Gemstone.GemPerpetualPositionAction
import GemstonePrimitives
import GemstonePrimitivesTestKit
@testable import Transfer

public extension AmountPerpetualViewModel {
    static func mock(
        action: GemPerpetualPositionAction = .open(data: .mock(direction: .long, price: 100, leverage: 3)),
        leverage: GemLeverageSelection? = nil,
    ) -> AmountPerpetualViewModel {
        AmountPerpetualViewModel(session: .mock(action: action, leverage: leverage, format: NumberInput.format()))
    }
}
