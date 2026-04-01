// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PriceService
import Primitives

public struct PriceUpdaterMock: PriceUpdater {
    public init() {}

    public func addPrices(assetIds _: [AssetId]) async throws {}
}

public extension PriceUpdater where Self == PriceUpdaterMock {
    static func mock() -> PriceUpdaterMock {
        PriceUpdaterMock()
    }
}
