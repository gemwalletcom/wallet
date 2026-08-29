// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import enum Gemstone.GemPortfolioDataInput
import protocol Gemstone.GemPortfolioServiceProtocol
import Primitives

public struct PortfolioDataService: Sendable {
    private let portfolioService: any GemPortfolioServiceProtocol

    public init(portfolioService: any GemPortfolioServiceProtocol) {
        self.portfolioService = portfolioService
    }

    func portfolioData(input: GemPortfolioDataInput) async throws -> PortfolioData {
        try await PortfolioData(portfolioService.portfolioData(input: input))
    }
}
