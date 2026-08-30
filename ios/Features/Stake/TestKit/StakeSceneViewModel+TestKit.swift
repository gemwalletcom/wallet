// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import Stake
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitivesTestKit

public extension StakeSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        chain: StakeChain = .tron,
        stakeService: any GemStakeServiceProtocol = GemStakeServiceMock(),
    ) -> StakeSceneViewModel {
        StakeSceneViewModel(
            wallet: wallet,
            chain: chain,
            currencyCode: "USD",
            stakeService: stakeService,
            explorerService: GemExplorerServiceMock(),
        )
    }
}
