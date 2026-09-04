// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemRecentActivityService
import Components
import GemstonePrimitivesTestKit
import protocol Gemstone.GemPerpetualServiceProtocol
import Store
import StoreTestKit
import GemstoneServices
import GemstoneServicesTestKit
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct PerpetualsSceneViewModelTests {
    @Test
    func headerViewModel() {
        let wallet = Wallet.mock(type: .multicoin)
        let model = PerpetualsSceneViewModel.mock(wallet: wallet)

        #expect(model.headerViewModel.walletType == .multicoin)
    }

    @Test
    func pullToRefreshUpdatesMarketsThatTheTimerWouldSkip() async {
        let perpetuals = GemPerpetualServiceMock()
        let model = PerpetualsSceneViewModel.mock(perpetualService: perpetuals)

        await model.load(source: .timer)
        #expect(perpetuals.syncMarketsCount == 1)

        await model.load(source: .timer)
        #expect(perpetuals.syncMarketsCount == 1)

        await model.load(source: .user)
        #expect(perpetuals.syncMarketsCount == 2)
    }

    @Test
    func openingTheSceneSyncsPositionsAndMarkets() async {
        let perpetuals = GemPerpetualServiceMock()
        let model = PerpetualsSceneViewModel.mock(perpetualService: perpetuals)

        await model.load()

        #expect(perpetuals.syncPositionsCount == 1)
        #expect(perpetuals.syncMarketsCount == 1)
    }
}

extension PerpetualsSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        observerService: any PerpetualObservable = PerpetualObserverMock(),
        recentAssetsService: any GemRecentActivityServiceProtocol = GemRecentActivityService(store: GemstoneRecentActivityStore(store: .mock()), session: .mock()),
    ) -> PerpetualsSceneViewModel {
        PerpetualsSceneViewModel(
            wallet: wallet,
            service: perpetualService,
            observerService: observerService,
            recentAssetsService: recentAssetsService,
        )
    }
}
