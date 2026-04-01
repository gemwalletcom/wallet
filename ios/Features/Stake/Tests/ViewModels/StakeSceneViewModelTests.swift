// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import StakeService
import StakeServiceTestKit
import StakeTestKit
import Testing

@testable import Stake
@testable import Store

@MainActor
struct StakeSceneViewModelTests {
    @Test
    func aprValue() throws {
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: 13.5)).stakeAprModel.subtitle.text == "13.50%")
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: 0)).stakeAprModel.subtitle.text == .empty)
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: .none)).stakeAprModel.subtitle.text == .empty)
    }

    @Test
    func testLockTimeField() throws {
        #expect(StakeSceneViewModel.mock(chain: .tron).lockTimeField.value.text == "14 days")
    }

    @Test
    func minimumStakeAmount() throws {
        #expect(StakeSceneViewModel.mock(chain: .tron).minAmountField?.value.text == "1.00 TRX")
    }

    @Test
    func showManage() throws {
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
}
