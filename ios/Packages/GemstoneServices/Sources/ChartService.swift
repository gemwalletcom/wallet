// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemChartService
import GemstonePrimitives
import Primitives

public struct ChartService: Sendable {
    private let service: GemChartService

    public init(service: GemChartService) {
        self.service = service
    }

    public func getCharts(assetId: AssetId, period: ChartPeriod) async throws -> Primitives.Charts {
        try await Primitives.Charts(
            service.getCharts(assetId: assetId.identifier, period: period.json()),
        )
    }
}
