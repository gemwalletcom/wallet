// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Stake

public extension EarnSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        asset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
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
