// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.PerpetualProvider
import typealias Gemstone.PerpetualData
import typealias Gemstone.PerpetualMarketData
import typealias Gemstone.PerpetualPosition
import protocol Gemstone.GemPerpetualStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstonePerpetualStore: GemPerpetualStore, @unchecked Sendable {
    private let store: PerpetualStore
    private let assetStore: AssetStore
    private let balanceStore: BalanceStore

    public init(store: PerpetualStore, assetStore: AssetStore, balanceStore: BalanceStore) {
        self.store = store
        self.assetStore = assetStore
        self.balanceStore = balanceStore
    }

    public func savePerpetuals(data: [Gemstone.PerpetualData]) async throws {
        let perpetualsData = try data.map { try Primitives.PerpetualData($0) }
        try assetStore.add(assets: perpetualsData.map { perpetualAssetBasic(from: $0.asset) })
        try store.upsertPerpetuals(perpetualsData.map(\.perpetual))
    }

    public func setPinned(perpetualIds: [String], pinned: Bool) async throws {
        try store.setPinned(for: perpetualIds, value: pinned)
    }

    public func clear() async throws {
        try store.clear()
        try balanceStore.deleteBalance(assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id)
    }

    public func getPositions(walletId: String, provider: Gemstone.PerpetualProvider) async throws -> [Gemstone.PerpetualPosition] {
        try store.getPositions(walletId: WalletId.from(id: walletId), provider: provider.map()).map { try $0.json() }
    }

    public func updateMarket(market: Gemstone.PerpetualMarketData) async throws {
        let market = try Primitives.PerpetualMarketData(market)
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

    public func getPositionIds(walletId: String, provider: Gemstone.PerpetualProvider) async throws -> [String] {
        try store.getPositions(walletId: WalletId.from(id: walletId), provider: provider.map()).map(\.id)
    }

    public func updatePositions(walletId: String, positions: [Gemstone.PerpetualPosition], deleteIds: [String]) async throws {
        try store.diffPositions(
            deleteIds: deleteIds,
            positions: positions.map { try Primitives.PerpetualPosition($0) },
            walletId: WalletId.from(id: walletId),
        )
    }

    private func perpetualAssetBasic(from asset: Asset) -> AssetBasic {
        AssetBasic(
            asset: asset,
            properties: AssetProperties(
                isEnabled: false,
                isBuyable: false,
                isSellable: false,
                isSwapable: false,
                isStakeable: false,
                stakingApr: nil,
                isEarnable: false,
                earnApr: nil,
                hasImage: false,
            ),
            score: AssetScore(rank: 0),
            price: nil,
        )
    }
}
