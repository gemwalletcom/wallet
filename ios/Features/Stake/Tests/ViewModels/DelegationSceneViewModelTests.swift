// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import Foundation
import Primitives
import PrimitivesTestKit
@testable import Stake
import StakeTestKit
import Testing

struct DelegationSceneViewModelTests {
    @Test
    func rewardsShownWhenCoreReportsThem() {
        let claimable = DelegationSceneViewModel.mock(stakeService: GemStakeServiceMock(claimable: true))
        let notClaimable = DelegationSceneViewModel.mock(stakeService: GemStakeServiceMock(claimable: false))

        #expect(claimable.canClaimRewards == true)
        #expect(notClaimable.canClaimRewards == false)
    }
}
