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

        await model.updateMarkets(source: .timer)
        #expect(perpetuals.syncMarketsCount == 1)

        await model.updateMarkets(source: .timer)
        #expect(perpetuals.syncMarketsCount == 1)

        await model.updateMarkets(source: .user)
        #expect(perpetuals.syncMarketsCount == 2)
    }
}

extension PerpetualsSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        observerService: any PerpetualObservable = PerpetualObserverMock(),
        recentAssetsService: any GemRecentActivityServiceProtocol = GemRecentActivityService(store: GemstoneRecentActivityStore(store: .mock())),
    ) -> PerpetualsSceneViewModel {
        PerpetualsSceneViewModel(
            wallet: wallet,
            service: perpetualService,
            observerService: observerService,
            recentAssetsService: recentAssetsService,
        )
    }
}
