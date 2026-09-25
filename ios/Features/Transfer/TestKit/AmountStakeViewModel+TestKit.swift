// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemStakeAmountInput
import class Gemstone.GemStakeService
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Transfer

public extension AmountStakeViewModel {
    static func mock(
        asset: Asset = .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18),
        type: GemStakeAmountInput = .stake(validator: DelegationValidator.mock().toGem()),
    ) -> AmountStakeViewModel {
        AmountStakeViewModel(asset: asset, type: type, service: GemStakeService.mock())
    }
}
