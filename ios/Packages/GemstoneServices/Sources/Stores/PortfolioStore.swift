// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPortfolioStore
import struct Gemstone.PortfolioAsset
import typealias Gemstone.WalletId
import GemstonePrimitives
import Primitives
import Store

public final class GemstonePortfolioStore: GemPortfolioStore, @unchecked Sendable {
    private let assetStore: AssetStore

    public init(assetStore: AssetStore) {
        self.assetStore = assetStore
    }

    public func getWalletAssets(walletId: Gemstone.WalletId) async throws -> [Gemstone.PortfolioAsset] {
        try assetStore.getAssetsData(walletId: Primitives.WalletId.from(id: walletId), filters: [.enabledBalance, .hasBalance], limit: nil)
            .map { Gemstone.PortfolioAsset(assetId: $0.asset.id.identifier, value: $0.balance.total.magnitude) }
    }
}
