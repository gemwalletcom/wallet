// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import Foundation
import Primitives
import PrimitivesTestKit
import Stake
import Testing

struct DelegationViewModelTests {
    @Test
    func balance() {
        let model = DelegationViewModel.mock()

        #expect(model.balanceText == "1,500 TRX")
        #expect(model.fiatValueText == "$3,000.00")
    }

    @Test
    func rewards() {
        let shown = DelegationViewModel.mock(rewardsShown: true)

        #expect(shown.rewardsText == "500 TRX")
        #expect(shown.rewardsFiatValueText == "$1,000.00")

        let hidden = DelegationViewModel.mock(rewardsShown: false)

        #expect(hidden.rewardsText == nil)
        #expect(hidden.rewardsFiatValueText == nil)
    }

    @Test
    func completionDate() {
        let completionDate = Date.now.addingTimeInterval(86400)

        #expect(DelegationViewModel.mock(state: .pending, completionDate: completionDate).completionDateText == "23 hours, 59 minutes")
        #expect(DelegationViewModel.mock(state: .active, completionDate: completionDate).completionDateText == nil)
    }
}

extension DelegationViewModel {
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
            currencyCode: "USD",
        )
    }
}
