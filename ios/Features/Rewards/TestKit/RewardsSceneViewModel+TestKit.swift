// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRewardsServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import Rewards

public extension RewardsSceneViewModel {
    static func mock(
        service: any GemRewardsServiceProtocol = GemRewardsServiceMock(),
        wallets: [Wallet],
        activateCode: String? = nil,
    ) -> RewardsSceneViewModel? {
        RewardsSceneViewModel(service: service, wallets: wallets, currentWallet: wallets.first, activateCode: activateCode)
    }
}
