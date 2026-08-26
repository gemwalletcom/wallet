// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemPriceService
import GemstonePrimitives
import Primitives

public struct MarketService: Sendable {
    private let service: GemPriceService

    public init(service: GemPriceService) {
        self.service = service
    }

    public func getMarkets() async throws -> Markets {
        try await Markets(service.getMarkets())
    }
}
