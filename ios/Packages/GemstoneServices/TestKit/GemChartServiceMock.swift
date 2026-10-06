// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import struct Gemstone.GemChart
import struct Gemstone.GemChartInput
import enum Gemstone.GemChartRequest
import struct Gemstone.GemChartResult
import protocol Gemstone.GemChartServiceProtocol
import struct Gemstone.GemChartSession
import struct Gemstone.GemChartView
import struct Gemstone.GemChartZoom
import GemstonePrimitivesTestKit

public final class GemChartServiceMock: GemChartServiceProtocol, @unchecked Sendable {
    private let charts: @Sendable (GemChartRequest) async -> GemChart
    public let requests: AsyncStream<GemChartRequest>
    private let requested: AsyncStream<GemChartRequest>.Continuation

    public init(charts: @escaping @Sendable (GemChartRequest) async -> GemChart = { _ in .mock() }) {
        self.charts = charts
        (requests, requested) = AsyncStream<GemChartRequest>.makeStream()
    }

    public func newSession() -> GemChartSession {
        GemChartSession(period: .day, currency: .usd, rate: .known(rate: 1), chart: nil, error: nil, isLoading: true, isRefreshing: false, zoom: GemChartZoom(scale: 1, offset: 0))
    }

    public func load(assetId _: AssetId, request: GemChartRequest) async -> GemChartResult {
        requested.yield(request)
        return await GemChartResult(request: request, rate: 1, state: .data, chart: charts(request))
    }

    public func viewState(session: GemChartSession, input _: GemChartInput) -> GemChartView {
        GemChartView(
            period: session.period,
            phase: session.isLoading ? .loading : session.chart.map { _ in .data(data: .mock()) } ?? .noData,
            isRefreshing: session.isRefreshing,
            sections: [],
        )
    }
}
