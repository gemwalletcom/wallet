// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Stake
import StakeTestKit
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
}
