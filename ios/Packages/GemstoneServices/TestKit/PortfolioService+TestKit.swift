// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPortfolioServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension PortfolioService {
    static func mock(
        service: any GemPortfolioServiceProtocol = GemPortfolioServiceMock(),
        assetStore: AssetStore = .mock(),
    ) -> PortfolioService {
        PortfolioService(
            service: service,
            assetStore: assetStore,
        )
    }
}
