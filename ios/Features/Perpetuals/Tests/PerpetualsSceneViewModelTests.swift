// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Localization
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct PerpetualsSceneViewModelTests {
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

    @Test
    func unavailablePerpetualsBlocksDepositButKeepsWithdrawal() {
        let service = GemPerpetualServiceMock()
        service.isAvailableValue = false
        var amounts: [AmountInput] = []
        let model = PerpetualsSceneViewModel.mock(onSelectAmount: { amounts.append($0) }, perpetualService: service)
        let asset = Asset.mock().toGem()

        #expect(model.isPresentingInfoSheet == nil)
        model.onSelectHeaderAction(.deposit(asset: asset))
        #expect(amounts.isEmpty)
        #expect(model.isPresentingInfoSheet?.description == Localized.Info.regionUnavailableDescription)

        model.isPresentingInfoSheet = nil
        model.onSelectHeaderAction(.withdraw(asset: asset))
        #expect(amounts.count == 1)
        #expect(model.isPresentingInfoSheet == nil)
        #expect(service.availabilityCheckCount == 1)

        service.isAvailableValue = true
        model.onSelectHeaderAction(.deposit(asset: asset))
        #expect(amounts.count == 2)
    }
}
