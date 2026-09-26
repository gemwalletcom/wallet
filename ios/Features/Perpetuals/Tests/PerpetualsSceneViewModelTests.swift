// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitivesTestKit
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

        #expect(model.header.isWatchWallet == false)
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
