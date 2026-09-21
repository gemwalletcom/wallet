// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Stake

public extension DelegationViewModel {
    static func mock(
        state: DelegationState = .active,
        rewardsShown: Bool = false,
    ) -> DelegationViewModel {
        DelegationViewModel(
            service: GemStakeServiceMock(rewardsShown: rewardsShown),
            delegation: .mock(
                state: state,
                price: Price.mock(price: 2.0),
                base: .mock(
                    state: state,
                    assetId: .mock(.tron),
                    balance: 1_500_000_000,
                    rewards: rewardsShown ? 500_000_000 : 0,
                ),
            ),
            asset: Chain.tron.asset,
            currency: .usd,
        )
    }
}
