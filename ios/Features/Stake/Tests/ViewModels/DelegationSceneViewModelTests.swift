// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemDelegationAction
import GemstonePrimitivesTestKit
import Localization
import Primitives
import PrimitivesTestKit
@testable import Stake
import StakeTestKit
import Testing

struct DelegationSceneViewModelTests {
    @Test
    func claimingRewardsNavigatesToConfirm() {
        var route: StakeRoute?
        let model = DelegationSceneViewModel.mock(
            rewards: 500_000,
            stakeService: GemStakeServiceMock(claimable: true),
            onNavigate: { route = $0 },
        )

        model.onClaimRewards()

        guard case .transfer(.confirm) = route else {
            Issue.record("expected a confirm route, got \(String(describing: route))")
            return
        }
    }

    @Test
    func claimingNothingIsNotATransfer() {
        var route: StakeRoute?
        let model = DelegationSceneViewModel.mock(rewards: 0, onNavigate: { route = $0 })

        model.onClaimRewards()

        #expect(route == nil, "there is nothing to claim, so there is no transfer to confirm")
    }

    @Test
    func anActionThatLandsOnTheDetailsStaysPut() {
        var route: StakeRoute?
        let model = DelegationSceneViewModel.mock(onNavigate: { route = $0 })

        model.onSelectAction(.unstake)

        #expect(route == nil)
    }

    @Test
    func rewardsShownWhenCoreReportsThem() {
        let claimable = DelegationSceneViewModel.mock(stakeService: GemStakeServiceMock(claimable: true))
        let notClaimable = DelegationSceneViewModel.mock(stakeService: GemStakeServiceMock(claimable: false))

        #expect(claimable.canClaimRewards == true)
        #expect(notClaimable.canClaimRewards == false)
    }

    @Test
    func rewardsRowOnlyWhenThereAreRewards() {
        let shown = DelegationSceneViewModel.mock(rewards: 500_000)
        let hidden = DelegationSceneViewModel.mock(rewards: 0)

        #expect(shown.rewardsItem?.title == Localized.Stake.rewards)
        #expect(shown.rewardsItem?.subtitle == "0.5 ATOM")
        #expect(hidden.rewardsItem == nil)
    }
}
