// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Stake
import StakeTestKit
import Testing

struct DelegationSceneViewModelTests {
    @Test
    func showManage() {
        #expect(DelegationSceneViewModel.mock(wallet: .mock(type: .multicoin)).showManage == true)
        #expect(DelegationSceneViewModel.mock(wallet: .mock(type: .view)).showManage == false)
    }

    @Test
    func availableActionsStake() {
        #expect(DelegationSceneViewModel.mock(wallet: .mock(type: .view)).availableActions == [])
        #expect(DelegationSceneViewModel.mock(chain: .cosmos, state: .active).availableActions == [.stake, .unstake, .redelegate])
        #expect(DelegationSceneViewModel.mock(chain: .ethereum, state: .active).availableActions == [.unstake])
        #expect(DelegationSceneViewModel.mock(chain: .cosmos, state: .inactive).availableActions == [.unstake, .redelegate])
        #expect(DelegationSceneViewModel.mock(chain: .tron, state: .awaitingWithdrawal).availableActions == [.withdraw])
        #expect(DelegationSceneViewModel.mock(chain: .cosmos, state: .awaitingWithdrawal).availableActions == [])
        #expect(DelegationSceneViewModel.mock(chain: .tron, state: .pending).availableActions == [])
        #expect(DelegationSceneViewModel.mock(chain: .tron, state: .activating).availableActions == [])
        #expect(DelegationSceneViewModel.mock(chain: .tron, state: .deactivating).availableActions == [])
    }

    @Test
    func availableActionsEarn() {
        #expect(DelegationSceneViewModel.mock(chain: .ethereum, state: .active, providerType: .earn).availableActions == [.deposit, .withdraw])
        #expect(DelegationSceneViewModel.mock(chain: .ethereum, state: .inactive, providerType: .earn).availableActions == [.withdraw])
        #expect(DelegationSceneViewModel.mock(chain: .ethereum, state: .pending, providerType: .earn).availableActions == [])
        #expect(DelegationSceneViewModel.mock(chain: .ethereum, state: .awaitingWithdrawal, providerType: .earn).availableActions == [])
    }

    @Test
    func canClaimRewards() {
        #expect(DelegationSceneViewModel.mock(chain: .cosmos, state: .active, rewards: "100").canClaimRewards == true)
        #expect(DelegationSceneViewModel.mock(chain: .ethereum, state: .active, rewards: "100").canClaimRewards == false)
        #expect(DelegationSceneViewModel.mock(chain: .cosmos, state: .inactive, rewards: "100").canClaimRewards == false)
        #expect(DelegationSceneViewModel.mock(chain: .cosmos, state: .active, rewards: "0").canClaimRewards == false)
        #expect(DelegationSceneViewModel.mock(wallet: .mock(type: .view), chain: .cosmos, state: .active, rewards: "100").canClaimRewards == false)
    }
}
