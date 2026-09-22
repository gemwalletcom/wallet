// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAssetBalance
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

    public func getWalletBalances(walletId: Gemstone.WalletId) async throws -> [Gemstone.GemAssetBalance] {
        try assetStore.getAssetsData(walletId: Primitives.WalletId.from(id: walletId), filters: [.enabledBalance, .hasBalance], limit: nil)
            .map { GemAssetBalance($0.balance, assetId: $0.asset.id, isActive: $0.metadata.isActive) }
    }
}
