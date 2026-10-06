// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import protocol Gemstone.GemChartServiceProtocol
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit

public extension ChartSceneViewModel {
    @MainActor
    static func mock(service: any GemChartServiceProtocol = GemChartServiceMock()) -> ChartSceneViewModel {
        ChartSceneViewModel(
            service: service,
            preferences: .mock(),
            asset: .mock(),
            onSetPriceAlert: { _ in },
        )
    }
}
