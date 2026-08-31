// Copyright (c). Gem Wallet. All rights reserved.

import typealias Gemstone.WalletId
import Foundation
import typealias Gemstone.AssetId
import typealias Gemstone.AssetMarket
import typealias Gemstone.Currency
import typealias Gemstone.FiatRate
import struct Gemstone.GemAssetPrice
import protocol Gemstone.GemPriceStore
import struct Gemstone.GemPriceUpdate
import GemstonePrimitives
import Primitives
import Store

public final class GemstonePriceStore: GemPriceStore, @unchecked Sendable {
    private let priceStore: PriceStore
    private let fiatRateStore: FiatRateStore

    public init(priceStore: PriceStore, fiatRateStore: FiatRateStore) {
        self.priceStore = priceStore
        self.fiatRateStore = fiatRateStore
    }

    public func getPrices(assetIds: [Gemstone.AssetId]) throws -> [GemAssetPrice] {
        try priceStore.getPrices(for: assetIds).map(\.gem)
    }

    public func getEnabledPriceAssetIds(walletId: Gemstone.WalletId) async throws -> [Gemstone.AssetId] {
        try priceStore.enabledPriceAssets(walletId: Primitives.WalletId.from(id: walletId)).map(\.identifier)
    }

    public func getRate(currency: Gemstone.Currency) async throws -> Gemstone.FiatRate? {
        let currency = try Primitives.Currency(currency)
        return try priceStore.getRate(currency: currency.rawValue).map { Primitives.FiatRate(symbol: currency, rate: $0.rate).json() }
    }

    public func saveRates(rates: [Gemstone.FiatRate]) async throws {
        try fiatRateStore.add(rates.map { try Primitives.FiatRate($0) })
    }

    public func savePrices(currency _: Gemstone.Currency, prices: [GemPriceUpdate]) async throws {
        try priceStore.updatePrices(prices.map { update in
            try PriceUpdate(
                assetId: Primitives.AssetId(id: update.assetId),
                price: update.price,
                priceUsd: update.priceUsd,
                priceChangePercentage24h: update.priceChangePercentage24h,
                updatedAt: Date(timeIntervalSince1970: TimeInterval(update.updatedAt)),
            )
        })
    }

    public func convertPrices(currency _: Gemstone.Currency, rate: Double) async throws {
        try priceStore.convertPrices(rate: rate)
    }

    public func saveMarket(assetId: Gemstone.AssetId, market: Gemstone.AssetMarket) async throws {
        try priceStore.updateMarket(assetId: assetId, market: Primitives.AssetMarket(market))
    }
}

private extension Primitives.AssetPrice {
    var gem: GemAssetPrice {
        GemAssetPrice(
            assetId: assetId.identifier,
            price: price,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: Int64(updatedAt.timeIntervalSince1970),
        )
    }
}
