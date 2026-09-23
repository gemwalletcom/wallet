// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemListRowTitle
import enum Gemstone.GemServiceError
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletTab
import WalletTabTestKit

@MainActor
struct PortfolioSceneViewModelTests {
    @Test
    func testShowSegmentedControl() {
        let service = GemPortfolioServiceMock()
        #expect(PortfolioSceneViewModel.mock(service: service, preferences: .mock(isPerpetualEnabled: false)).showSegmentedControl == false)
        #expect(PortfolioSceneViewModel.mock(service: service, preferences: .mock(isPerpetualEnabled: true)).showSegmentedControl == false, "a wallet without a perpetual chain has one portfolio")

        service.perpetualsShown = true
        #expect(PortfolioSceneViewModel.mock(service: service, preferences: .mock(isPerpetualEnabled: true)).showSegmentedControl)
        #expect(PortfolioSceneViewModel.mock(service: service, preferences: .mock(isPerpetualEnabled: false)).showSegmentedControl == false, "the setting still hides it")
    }

    @Test
    func testShowChartTypePicker() {
        #expect(PortfolioSceneViewModel.mock(defaultType: .wallet).showChartTypePicker == false)
        #expect(PortfolioSceneViewModel.mock(defaultType: .perpetuals).showChartTypePicker == true)
    }

    @Test
    func navigationTitleChangesWithType() {
        let model = PortfolioSceneViewModel.mock()
        let walletTitle = model.navigationTitle

        model.selectedType = Primitives.PortfolioType.perpetuals

        #expect(walletTitle != model.navigationTitle)
    }

    @Test
    func eachTypeCarriesItsOwnStatistics() async {
        let model = PortfolioSceneViewModel.mock(service: GemPortfolioServiceMock())
        #expect(model.statisticRows.isEmpty)

        await model.load()
        #expect(model.statisticRows.count == 2)

        model.selectedType = Primitives.PortfolioType.perpetuals
        await model.loadIfNeeded()
        #expect(model.statisticRows.count == 5)
    }

    @Test
    func theOfferedPeriodsComeFromThePortfolio() async {
        let service = GemPortfolioServiceMock()
        service.dataForType = { _ in .mockWallet(availablePeriods: [.day, .month]) }
        let model = PortfolioSceneViewModel.mock(service: service)

        #expect(model.periods == [.day, .week, .month, .year, .all])

        await model.load()

        #expect(model.periods == [.day, .month])
        #expect(model.selectedPeriod == .day, "the portfolio does not offer the selected period, so it falls back to the first")
    }

    @Test
    func testChartState() async {
        let model = PortfolioSceneViewModel.mock()
        #expect(model.chartState.isLoading)

        await model.load()
        #expect(!model.chartState.isLoading)
    }

    @Test
    func aFailedLoadShowsTheError() async {
        let service = GemPortfolioServiceMock()
        service.error = .Core(msg: "offline")
        let model = PortfolioSceneViewModel.mock(service: service)

        await model.load()

        #expect(model.chartState.isError)
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

    @Test
    func theStatisticRowsCarryTheTitlesCoreChose() async {
        let model = PortfolioSceneViewModel.mock()
        model.selectedType = Primitives.PortfolioType.perpetuals

        await model.load()

        let titles: [GemListRowTitle] = model.statisticRows.compactMap { row in
            switch row {
            case let .amount(title, _, _): title
            case let .label(title, _, _, _, _): title
            case let .allTime(title, _, _, _): title
            default: nil
            }
        }

        #expect(titles.contains(.unrealizedPnl))
        #expect(titles.contains(.accountLeverage))
        #expect(titles.contains(.marginUsage))
        #expect(titles.count == model.statisticRows.count, "every statistic became a row Core filled")
    }
}
