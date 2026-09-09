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

        #expect(model.lockTimeField.value.text == "14 days")
    }

    @Test
    func minimumStakeAmount() {
        let model = StakeSceneViewModel.mock(chain: .tron, stakeService: GemStakeServiceMock(minStake: 1_000_000))

        #expect(model.minAmountField?.value.text == "1 TRX")
    }

    @Test
    func chainWithoutAMinimumHasNoField() {
        #expect(StakeSceneViewModel.mock(chain: .tron).minAmountField == nil)
    }

    @Test
    func stakeStillRequiresValidators() {
        let tron = StakeSceneViewModel.mock(chain: .tron)
        tron.assetQuery.value = .mock(asset: Chain.tron.asset, balance: .mock(frozen: 1))

        #expect(tron.isStakeEnabled == false)
        #expect(tron.stakeInfoAction == nil)
    }

}
