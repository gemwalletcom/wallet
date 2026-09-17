// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import PrimitivesTestKit
@testable import Stake
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import StakeTestKit
@testable import Store
import Testing

@MainActor
struct StakeSceneViewModelTests {
    @Test
    func testLockTimeField() {
        let model = StakeSceneViewModel.mock(chain: .tron, stakeService: GemStakeServiceMock(lockTime: 1_209_600))

        #expect(model.infoField(for: .lockTime).value.text == "14 days")
    }

    @Test
    func minimumStakeAmount() {
        let model = StakeSceneViewModel.mock(chain: .tron, stakeService: GemStakeServiceMock(minStake: 1_000_000))

        #expect(model.infoField(for: .minimumAmount).value.text == "1 TRX")
        #expect(model.infoRows.contains(.minimumAmount))
    }

    @Test
    func chainWithoutAMinimumHasNoField() {
        #expect(StakeSceneViewModel.mock(chain: .tron).infoRows.contains(.minimumAmount) == false)
    }

    @Test
    func stakeStillRequiresValidators() {
        let tron = StakeSceneViewModel.mock(chain: .tron)
        tron.assetQuery.value = .mock(asset: Chain.tron.asset, balance: .mock(frozen: 1))

        let stake = tron.actions.first { $0.action == .stake }
        #expect(stake?.isEnabled == false)
        #expect(stake?.requiresFrozenBalance == false)
    }

}
