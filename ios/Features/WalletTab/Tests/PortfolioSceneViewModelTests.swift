// Copyright (c). Gem Wallet. All rights reserved.

import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletTab
import WalletTabTestKit

@MainActor
struct PortfolioSceneViewModelTests {
    @Test
    func testShowSegmentedControl() {
        #expect(PortfolioSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: false)).showSegmentedControl == false)
        #expect(PortfolioSceneViewModel.mock(preferences: .mock(isPerpetualEnabled: true)).showSegmentedControl == false)
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

        model.state.selectedType = .perpetuals
        let perpTitle = model.navigationTitle

        #expect(walletTitle != perpTitle)
    }

    @Test
    func testStatistics() {
        let model = PortfolioSceneViewModel.mock()
        #expect(model.statistics.isEmpty)

        model.state.wallet = .data(.mockWallet())
        #expect(model.statistics.count == 2)

        model.state.selectedType = .perpetuals
        model.state.perpetual = .data(.mockPerpetual())
        #expect(model.statistics.count == 5)
    }

    @Test
    func testPeriods() {
        let model = PortfolioSceneViewModel.mock()
        #expect(model.periods == [.day, .week, .month, .year, .all])

        model.state.wallet = .data(.mockWallet(availablePeriods: [.day, .month]))
        #expect(model.periods == [.day, .month])
    }

    @Test
    func testChartState() {
        let model = PortfolioSceneViewModel.mock()
        #expect(model.chartState.isLoading)

        model.state.wallet = .data(.mockWallet())
        #expect(!model.chartState.isLoading)
        #expect(!model.chartState.isNoData)

        model.state.wallet = .error(AnyError("test"))
        #expect(model.chartState.isError)
    }

    @Test
    func onTypeChangedSkipsFetchWhenCached() {
        let model = PortfolioSceneViewModel.mock()
        model.state.perpetual = .data(.mockPerpetual())
        model.state.selectedType = .perpetuals

        model.onTypeChanged(.wallet, .perpetuals)

        #expect(!model.state.perpetual.isLoading)
    }

    @Test
    func onTypeChangedFetchesWhenNotCached() {
        let model = PortfolioSceneViewModel.mock()
        model.state.selectedType = .perpetuals

        model.onTypeChanged(.wallet, .perpetuals)

        #expect(model.state.perpetual.isLoading)
    }

    @Test
    func testStatisticModel() {
        let model = PortfolioSceneViewModel.mock()

        #expect(model.statisticModel(.allTimeHigh(.mock())).title == "All Time High")
        #expect(model.statisticModel(.allTimeLow(.mock())).title == "All Time Low")
        #expect(model.statisticModel(.unrealizedPnl(500)).title == "Unrealized PnL")
        #expect(model.statisticModel(.accountLeverage(2.5)).title == "Account Leverage")
        #expect(model.statisticModel(.accountLeverage(2.5)).subtitle == "2.50x")
        #expect(model.statisticModel(.marginUsage(.mock())).title == "Margin Usage")
        #expect(model.statisticModel(.allTimePnl(1200)).title == "All Time PnL")
        #expect(model.statisticModel(.volume(50000)).title == "Volume")
    }
}
