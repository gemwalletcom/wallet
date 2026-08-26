// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import PrimitivesTestKit
@testable import Stake
import GemstoneServices
import GemstoneServicesTestKit
import StakeTestKit
@testable import Store
import Testing

@MainActor
struct StakeSceneViewModelTests {
    @Test
    func testLockTimeField() {
        #expect(StakeSceneViewModel.mock(chain: .tron).lockTimeField.value.text == "14 days")
    }

    @Test
    func minimumStakeAmount() {
        #expect(StakeSceneViewModel.mock(chain: .tron).minAmountField?.value.text == "1 TRX")
    }

    @Test
    func showManage() {
        #expect(StakeSceneViewModel.mock(wallet: .mock(type: .multicoin)).showManage == true)
        #expect(StakeSceneViewModel.mock(wallet: .mock(type: .view)).showManage == false)
    }

    @Test
    func stakeRequiresFrozenResourcesOnTron() {
        let tron = StakeSceneViewModel.mock(chain: .tron)
        tron.validatorsQuery.value = [.mock(.tron)]

        tron.assetQuery.value = .mock(asset: Chain.tron.asset, balance: .mock())
        #expect(tron.isStakeEnabled == false)
        #expect(tron.stakeInfoAction != nil)

        tron.assetQuery.value = .mock(asset: Chain.tron.asset, balance: .mock(frozen: 1))
        #expect(tron.isStakeEnabled == true)
        #expect(tron.stakeInfoAction == nil)

        tron.assetQuery.value = .mock(asset: Chain.tron.asset, balance: .mock(locked: 1))
        #expect(tron.isStakeEnabled == true)
        #expect(tron.stakeInfoAction == nil)
    }

    @Test
    func stakeStillRequiresValidators() {
        let tron = StakeSceneViewModel.mock(chain: .tron)
        tron.assetQuery.value = .mock(asset: Chain.tron.asset, balance: .mock(frozen: 1))

        #expect(tron.isStakeEnabled == false)
        #expect(tron.stakeInfoAction == nil)
    }

    @Test
    func stakeNotGatedByFrozenResourcesOffTron() {
        let cosmos = StakeSceneViewModel.mock(chain: .cosmos)
        cosmos.validatorsQuery.value = [.mock(.cosmos)]
        cosmos.assetQuery.value = .mock(asset: Chain.cosmos.asset, balance: .mock())

        #expect(cosmos.isStakeEnabled == true)
        #expect(cosmos.stakeInfoAction == nil)
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
