// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Stake

public extension StakeSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        chain: StakeChain = .tron,
        stakeService: any GemStakeServiceProtocol = GemStakeServiceMock(),
        onNavigate: StakeRouteAction = nil,
    ) -> StakeSceneViewModel {
        StakeSceneViewModel(
            wallet: wallet,
            chain: chain,
            service: stakeService,
            onNavigate: onNavigate,
        )
    }
}
