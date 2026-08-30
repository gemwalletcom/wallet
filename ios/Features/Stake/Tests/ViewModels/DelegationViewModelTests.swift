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
        let model = DelegationViewModel.mock()

        #expect(model.rewardsText == "500 TRX")
        #expect(model.rewardsFiatValueText == "$1,000.00")

        let deactivating = DelegationViewModel.mock(state: .deactivating)

        #expect(deactivating.rewardsText == nil)
        #expect(deactivating.rewardsFiatValueText == nil)
    }

    @Test
    func completionDate() {
        let completionDate = Date.now.addingTimeInterval(86400)

        #expect(
            DelegationViewModel
                .mock(state: .deactivating, completionDate: completionDate)
                .completionDateText == "23 hours, 59 minutes",
        )
        #expect(DelegationViewModel.mock(state: .active, completionDate: completionDate).completionDateText == nil)
    }
}

extension DelegationViewModel {
    static func mock(
        state: DelegationState = .active,
        completionDate: Date? = nil,
    ) -> DelegationViewModel {
        DelegationViewModel(
            explorerService: GemExplorerServiceMock(),
            stakeConfig: GemStakeServiceMock().config(),
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
