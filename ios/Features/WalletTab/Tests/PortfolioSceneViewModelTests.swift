// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemListRowTitle
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
    func statistics() {
        let model = PortfolioSceneViewModel.mock()
        #expect(model.statisticRows.isEmpty)

        model.state.wallet = .data(.mockWallet())
        #expect(model.statisticRows.count == 2)

        model.state.selectedType = .perpetuals
        model.state.perpetual = .data(.mockPerpetual())
        #expect(model.statisticRows.count == 5)
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
    func theStatisticRowsCarryTheTitlesCoreChose() {
        let model = PortfolioSceneViewModel.mock()
        model.state.selectedType = .perpetuals
        model.state.perpetual = .data(.mockPerpetual())

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
