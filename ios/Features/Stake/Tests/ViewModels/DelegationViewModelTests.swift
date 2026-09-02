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

        #expect(
            DelegationViewModel
                .mock(completionDate: completionDate, completionDateShown: true)
                .completionDateText == "23 hours, 59 minutes",
        )
        #expect(DelegationViewModel.mock(completionDate: completionDate, completionDateShown: false).completionDateText == nil)
    }
}

extension DelegationViewModel {
    static func mock(
        state: DelegationState = .active,
        completionDate: Date? = nil,
        rewardsShown: Bool = false,
        completionDateShown: Bool = false,
    ) -> DelegationViewModel {
        DelegationViewModel(
            stakeService: GemStakeServiceMock(rewardsShown: rewardsShown, completionDateShown: completionDateShown),
            delegation: .mock(
                state: state,
                price: Price.mock(price: 2.0),
                base: .mock(
                    state: state,
                    assetId: .mock(.tron),
                    balance: "1500000000",
                    rewards: "500000000",
                    completionDate: completionDate,
                ),
            ),
            asset: Chain.tron.asset,
            currencyCode: "USD",
        )
    }
}
