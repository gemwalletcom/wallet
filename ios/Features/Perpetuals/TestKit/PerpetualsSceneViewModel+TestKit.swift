// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPerpetualServiceProtocol
import class Gemstone.GemRecentActivityService
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Perpetuals
import Primitives
import PrimitivesTestKit
import StoreTestKit

public extension PerpetualsSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
    ) -> PerpetualsSceneViewModel {
        PerpetualsSceneViewModel(
            wallet: wallet,
            service: perpetualService,
            observerService: PerpetualObserverMock(),
            recentAssetsService: GemRecentActivityService(store: GemstoneRecentActivityStore(store: .mock()), session: .mock()),
        )
    }
}
