// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDelegationActionItem
import GemstonePrimitivesTestKit
import Localization
import Primitives
import PrimitivesTestKit
@testable import Stake
import StakeTestKit
import Testing

@MainActor
struct DelegationSceneViewModelTests {
    @Test
    func claimingRewardsNavigatesToConfirm() {
        var route: StakeRoute?
        let model = DelegationSceneViewModel.mock(
            rewards: 500_000,
            stakeService: GemStakeServiceMock(claimable: true),
            onNavigate: { route = $0 },
        )

        model.details.claim.map(model.onClaimRewards)

        guard case .transfer(.confirm) = route else {
            Issue.record("expected a confirm route, got \(String(describing: route))")
            return
        }
    }

    @Test
    func claimingNothingIsNotATransfer() {
        var route: StakeRoute?
        let model = DelegationSceneViewModel.mock(rewards: 0, onNavigate: { route = $0 })

        model.details.claim.map(model.onClaimRewards)

        #expect(model.details.claim == nil)
        #expect(route == nil, "there is nothing to claim, so there is no transfer to confirm")
    }

    @Test
    func anActionThatLandsOnTheDetailsStaysPut() {
        var route: StakeRoute?
        let model = DelegationSceneViewModel.mock(onNavigate: { route = $0 })

        model.onSelectAction(GemDelegationActionItem(action: .unstake, destination: .details))

        #expect(route == nil)
    }

    @Test
    func rewardsShownWhenCoreReportsThem() {
        let claimable = DelegationSceneViewModel.mock(rewards: 500_000, stakeService: GemStakeServiceMock(claimable: true))
        let notClaimable = DelegationSceneViewModel.mock(rewards: 500_000, stakeService: GemStakeServiceMock(claimable: false))

        #expect(claimable.details.claim != nil)
        #expect(notClaimable.details.claim == nil)
    }

    @Test
    func rewardsRowOnlyWhenThereAreRewards() {
        let shown = DelegationSceneViewModel.mock(rewards: 500_000)
        let hidden = DelegationSceneViewModel.mock(rewards: 0)

        #expect(shown.rewardsItem(shown.details)?.title == Localized.Stake.rewards)
        #expect(shown.rewardsItem(shown.details)?.subtitle == "0.5 ATOM")
        #expect(hidden.rewardsItem(hidden.details) == nil)
    }

    @Test
    func theValidatorRowOpensAddressDetailsOnItsChain() {
        var selected: ChainAddress?
        let model = DelegationSceneViewModel.mock(chain: .cosmos, onSelectAddress: { selected = $0 })

        model.onSelectProvider?("cosmosvaloper1")

        #expect(selected == ChainAddress(chain: .cosmos, address: "cosmosvaloper1"))
        #expect(DelegationSceneViewModel.mock().onSelectProvider == nil)
    }
}
