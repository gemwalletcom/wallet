// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPortfolioServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct PortfolioService: Sendable {
    private let apiService: any GemPortfolioServiceProtocol
    private let assetStore: AssetStore

    public init(apiService: any GemPortfolioServiceProtocol, assetStore: AssetStore) {
        self.apiService = apiService
        self.assetStore = assetStore
    }

    public func getPortfolioAssets(walletId: WalletId, period: ChartPeriod) async throws -> PortfolioAssets {
        let assets = try assetStore.getAssetsData(walletId: walletId, filters: [.enabledBalance, .hasBalance])
        let portfolioAssets = assets.map { PortfolioAsset(assetId: $0.asset.id, value: String($0.balance.total)) }
        let request = PortfolioAssetsRequest(assets: portfolioAssets)
        return try await PortfolioAssets(apiService.getAssets(period: period.json(), request: request.json()))
    }
}
