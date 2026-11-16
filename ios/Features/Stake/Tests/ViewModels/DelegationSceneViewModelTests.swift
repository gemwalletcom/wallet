// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDelegationActionItem
import struct Gemstone.GemFormattedNumber
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
            stakeService: GemStakeServiceMock(details: .mock(claim: .mock())),
            onNavigate: { route = $0 },
        )

        model.details.claim.map(model.onClaimRewards)

        guard case .transfer(.confirm) = route else {
            Issue.record("expected a confirm route, got \(String(describing: route))")
            return
        }
    }

    @Test
    func anActionThatLandsOnTheDetailsStaysPut() {
        var route: StakeRoute?
        let model = DelegationSceneViewModel.mock(onNavigate: { route = $0 })

        model.onSelectAction(GemDelegationActionItem(action: .unstake, target: .open(destination: .details)))

        #expect(route == nil)
    }

    @Test
    func anActionUnavailableInTheRegionExplainsInsteadOfOpening() {
        var route: StakeRoute?
        let model = DelegationSceneViewModel.mock(onNavigate: { route = $0 })

        model.onSelectAction(GemDelegationActionItem(action: .stake, target: .info(topic: .regionUnavailable(country: "GB"))))

        #expect(route == nil)
        #expect(model.isPresentingInfoSheet?.description == .regionUnavailable(country: "GB"))
    }

    @Test
    func rewardsRowOnlyWhenThereAreRewards() {
        let rewards = GemFormattedNumber.mock(value: 0.5, unit: .symbol(symbol: "ATOM"), display: .number(precision: .significant(max: 4)))
        let shown = DelegationSceneViewModel.mock(stakeService: GemStakeServiceMock(details: .mock(rewards: rewards)))
        let hidden = DelegationSceneViewModel.mock()

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
