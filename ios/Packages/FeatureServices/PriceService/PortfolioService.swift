// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPortfolioServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct PortfolioService: Sendable {
    private let service: any GemPortfolioServiceProtocol
    private let assetStore: AssetStore

    public init(service: any GemPortfolioServiceProtocol, assetStore: AssetStore) {
        self.service = service
        self.assetStore = assetStore
    }

    public func getPortfolioAssets(walletId: WalletId, period: ChartPeriod) async throws -> PortfolioAssets {
        let assets = try assetStore.getAssetsData(walletId: walletId, filters: [.enabledBalance, .hasBalance])
        let portfolioAssets = assets.map { PortfolioAsset(assetId: $0.asset.id, value: String($0.balance.total)) }
        let request = PortfolioAssetsRequest(assets: portfolioAssets)
        return try await PortfolioAssets(service.getAssets(period: period.json(), request: request.json()))
    }
}
