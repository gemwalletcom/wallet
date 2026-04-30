// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import PrimitivesTestKit
@testable import Stake
import StakeService
import StakeServiceTestKit
import StakeTestKit
@testable import Store
import Testing

@MainActor
struct StakeSceneViewModelTests {
    @Test
    func aprValue() {
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: 13.5)).stakeAprModel.subtitle.text == "13.50%")
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: 0)).stakeAprModel.subtitle.text == .empty)
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: .none)).stakeAprModel.subtitle.text == .empty)
    }

    @Test
    func testLockTimeField() {
        #expect(StakeSceneViewModel.mock(chain: .tron).lockTimeField.value.text == "14 days")
    }

    @Test
    func minimumStakeAmount() {
        #expect(StakeSceneViewModel.mock(chain: .tron).minAmountField?.value.text == "1.00 TRX")
    }

    @Test
    func showManage() {
        #expect(StakeSceneViewModel.mock(wallet: .mock(type: .multicoin)).showManage == true)
        #expect(StakeSceneViewModel.mock(wallet: .mock(type: .view)).showManage == false)
    }

    @Test
    func recommendedCurrentValidator() throws {
        let model = StakeSceneViewModel.mock(chain: .cosmos)
        let recommendedId = try #require(StakeRecommendedValidators().validatorsSet(chain: .cosmos).first)

        model.validatorsQuery.value = [.mock(.cosmos, id: "other"), .mock(.cosmos, id: recommendedId)]

        #expect(model.recommendedCurrentValidator?.id == recommendedId)
    }

    @Test
    func rewardsState() {
        let oneReward = [Delegation.mock(base: .mock(state: .active, rewards: "100"))]
        let twoRewards = [
            Delegation.mock(validator: .mock(.monad, id: "a"), base: .mock(state: .active, rewards: "100")),
            Delegation.mock(validator: .mock(.monad, id: "b"), base: .mock(state: .active, rewards: "100")),
        ]

        let monadMulti = StakeSceneViewModel.mock(chain: .monad)
        monadMulti.delegationsQuery.value = twoRewards
        #expect(monadMulti.showRewards == true)
        #expect(monadMulti.canClaimAllRewards == false)

        let monadSingle = StakeSceneViewModel.mock(chain: .monad)
        monadSingle.delegationsQuery.value = oneReward
        #expect(monadSingle.canClaimAllRewards == true)

        let cosmos = StakeSceneViewModel.mock(chain: .cosmos)
        cosmos.delegationsQuery.value = oneReward
        #expect(cosmos.showRewards == true)
        #expect(cosmos.canClaimAllRewards == true)

        #expect(StakeSceneViewModel.mock(chain: .cosmos).showRewards == false)
    }
}
