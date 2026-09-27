// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.AssetData
import protocol Gemstone.GemPortfolioStore
import typealias Gemstone.WalletId
import GemstonePrimitives
import Primitives
import Store

public final class GemstonePortfolioStore: GemPortfolioStore, @unchecked Sendable {
    private let assetStore: AssetStore

    public init(assetStore: AssetStore) {
        self.assetStore = assetStore
    }

    public func getPortfolioAssets(walletId: Gemstone.WalletId) async throws -> [Gemstone.AssetData] {
        try assetStore.getAssetsData(walletId: Primitives.WalletId.from(id: walletId), filters: [.enabledBalance, .hasBalance], limit: nil)
            .map { $0.toGem() }
    }
}
