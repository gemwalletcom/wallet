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
    func stakeStillRequiresValidators() {
        let tron = StakeSceneViewModel.mock(chain: .tron)
        tron.assetQuery.value = .mock(asset: Chain.tron.asset, balance: .mock(frozen: 1))

        #expect(tron.isStakeEnabled == false)
        #expect(tron.stakeInfoAction == nil)
    }

}
