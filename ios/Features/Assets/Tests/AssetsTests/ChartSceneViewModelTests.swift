// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import AssetsTestKit
import struct Gemstone.GemChart
import enum Gemstone.GemChartRequest
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import Testing

@MainActor
struct ChartSceneViewModelTests {
    @Test(.timeLimit(.minutes(1)))
    func aPeriodPickedWhileALoadIsInFlightWins() async {
        let (charts, chart) = AsyncStream<GemChart>.makeStream()
        let service = GemChartServiceMock(charts: { _ in await charts.first { _ in true } ?? .mock() })
        let model = ChartSceneViewModel.mock(service: service)
        var requests = service.requests.makeAsyncIterator()

        let load = Task { await model.load() }
        #expect(await requests.next()?.period == .day)

        model.selectedPeriod = .week
        chart.yield(.mock())

        #expect(await requests.next()?.period == .week, "the answer for the period the user left is dropped and the new period loads")
        chart.yield(.mock())
        await load.value

        #expect(model.selectedPeriod == .week)
        #expect(model.chartState.isLoading == false)
    }
}

private extension GemChartRequest {
    var period: ChartPeriod? {
        switch self {
        case .rate: nil
        case let .chart(period, _): period.toPrimitives()
        }
    }
}
