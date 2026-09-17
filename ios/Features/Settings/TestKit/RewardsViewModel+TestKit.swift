// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRewardsServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import Settings

public extension RewardsViewModel {
    static func mock(
        service: any GemRewardsServiceProtocol = GemRewardsServiceMock(),
        wallets: [Wallet],
        activateCode: String? = nil,
    ) -> RewardsViewModel? {
        RewardsViewModel(service: service, wallets: wallets, currentWallet: wallets.first, activateCode: activateCode)
    }
}
