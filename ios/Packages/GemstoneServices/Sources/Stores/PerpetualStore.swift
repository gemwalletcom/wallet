// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import protocol Gemstone.GemPerpetualStore
import struct Gemstone.Perpetual
import struct Gemstone.PerpetualData
import struct Gemstone.PerpetualMarketData
import struct Gemstone.PerpetualPosition
import enum Gemstone.PerpetualProvider
import GemstonePrimitives
import Primitives
import Store

public final class GemstonePerpetualStore: GemPerpetualStore, @unchecked Sendable {
    private let store: PerpetualStore

    public init(store: PerpetualStore) {
        self.store = store
    }

    public func getPerpetuals(names: [String]) async throws -> [Gemstone.Perpetual] {
        try store.getPerpetuals(names: names).map { $0.toGem() }
    }

    public func savePerpetuals(data: [Gemstone.PerpetualData]) async throws {
        try store.upsertPerpetuals(data.map { $0.toPrimitives().perpetual })
    }

    public func setPinned(perpetualIds: [String], pinned: Bool) async throws {
        try store.setPinned(for: perpetualIds, value: pinned)
    }

    public func clearPerpetuals(collateralAssetIds: [Gemstone.AssetId]) async throws {
        try store.clear(collateralAssetIds: collateralAssetIds)
    }

    public func getPositions(walletId: WalletId, provider: Gemstone.PerpetualProvider) async throws -> [Gemstone.PerpetualPosition] {
        try store.getPositions(walletId: walletId, provider: provider.toPrimitives()).map { $0.toGem() }
    }

    public func updateMarket(market: Gemstone.PerpetualMarketData) async throws {
        try store.updateMarket(
            coin: market.coin,
            price: market.price,
            pricePercentChange24h: market.pricePercentChange24h,
            openInterest: market.openInterest,
            volume24h: market.volume24h,
            funding: market.funding,
        )
    }

    public func updatePrices(prices: [String: Double]) async throws {
        try store.updatePrices(prices)
    }

    public func getPositionIds(walletId: WalletId, provider: Gemstone.PerpetualProvider) async throws -> [String] {
        try store.getPositions(walletId: walletId, provider: provider.toPrimitives()).map(\.id)
    }

    public func updatePositions(walletId: WalletId, positions: [Gemstone.PerpetualPosition], deleteIds: [String]) async throws {
        try store.diffPositions(
            deleteIds: deleteIds,
            positions: positions.map { $0.toPrimitives() },
            walletId: walletId,
        )
    }
}
