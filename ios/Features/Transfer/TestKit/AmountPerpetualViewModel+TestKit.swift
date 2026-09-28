// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import enum Gemstone.GemPerpetualPositionAction
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Transfer

public extension AmountPerpetualViewModel {
    static func mock(
        action: GemPerpetualPositionAction = .open(data: .mock(direction: .long, price: 100, leverage: 3)),
        service: GemAmountServiceMock = GemAmountServiceMock(builder: GemAmountService.mock()),
    ) -> AmountPerpetualViewModel {
        AmountPerpetualViewModel(asset: .mock(), action: action, service: service)
    }
}
