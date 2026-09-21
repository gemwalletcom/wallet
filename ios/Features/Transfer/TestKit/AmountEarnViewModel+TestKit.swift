// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.EarnType
import class Gemstone.GemAmountService
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Transfer

public extension AmountEarnViewModel {
    static func mock(
        action: Gemstone.EarnType = .deposit(DelegationValidator.mock().toGem()),
    ) -> AmountEarnViewModel {
        AmountEarnViewModel(asset: .mock(), action: action, service: GemAmountService.mock())
    }
}
