// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Wallet
import WalletTestKit

@MainActor
struct PortfolioSceneViewModelTests {
    @Test
    func testChartState() async {
        let model = PortfolioSceneViewModel.mock()
        #expect(model.chartPhase == .loading)

        await model.load()
        #expect(model.chartPhase != .loading)
    }

    @Test
    func onlyAnOfflineLoadShowsTheError() async {
        let offlineService = GemPortfolioServiceMock()
        offlineService.error = .Offline
        let offline = PortfolioSceneViewModel.mock(service: offlineService)
        await offline.load()

        let failedService = GemPortfolioServiceMock()
        failedService.error = .Core(msg: "Not found")
        let failed = PortfolioSceneViewModel.mock(service: failedService)
        await failed.load()

        #expect(offline.chartPhase == .failed(error: .Offline))
        #expect(failed.chartPhase == .noData)
    }

    @Test
    func switchingBackToALoadedTypeAsksForNothing() async {
        let service = GemPortfolioServiceMock()
        let model = PortfolioSceneViewModel.mock(service: service)
        await model.load()

        model.selectedType = Primitives.PortfolioType.perpetuals
        await model.loadIfNeeded()
        model.selectedType = Primitives.PortfolioType.wallet
        await model.loadIfNeeded()

        #expect(service.requests.map(\.portfolioType) == [.wallet, .perpetuals])
    }
}
