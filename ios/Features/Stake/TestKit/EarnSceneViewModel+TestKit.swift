// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Stake

public extension EarnSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        asset: Asset = .mockEthereum(),
        stakeService: any GemStakeServiceProtocol = GemStakeServiceMock(),
        onNavigate: StakeRouteAction = nil,
    ) -> EarnSceneViewModel {
        EarnSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: stakeService,
            onNavigate: onNavigate,
        )
    }
}
