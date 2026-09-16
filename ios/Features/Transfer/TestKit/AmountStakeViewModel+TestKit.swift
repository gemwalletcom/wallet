// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import enum Gemstone.GemStakeAmountInput
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Transfer

public extension AmountStakeViewModel {
    static func mock(
        asset: Asset = .mockBNB(),
        type: GemStakeAmountInput = .stake(validators: [DelegationValidator.mock().toGem()], validator: nil),
    ) -> AmountStakeViewModel {
        AmountStakeViewModel(asset: asset, type: type, service: GemAmountService.mock())
    }
}
