// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Stake

public extension DelegationViewModel {
    static func mock(
        state: DelegationState = .active,
        completionDate: Date? = nil,
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
                    rewards: 500_000_000,
                    completionDate: completionDate,
                ),
            ),
            asset: Chain.tron.asset,
            currency: .usd,
        )
    }
}
