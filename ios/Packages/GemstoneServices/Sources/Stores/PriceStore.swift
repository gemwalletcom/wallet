// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import struct Gemstone.AssetMarket
import struct Gemstone.AssetPrice
import enum Gemstone.Currency
import struct Gemstone.FiatRate
import protocol Gemstone.GemPriceStore
import struct Gemstone.GemPriceUpdate
import GemstonePrimitives
import Primitives
import Store

public final class GemstonePriceStore: GemPriceStore, @unchecked Sendable {
    private let priceStore: PriceStore

    public init(priceStore: PriceStore) {
        self.priceStore = priceStore
    }

    public func getPrices(assetIds: [Gemstone.AssetId]) throws -> [Gemstone.AssetPrice] {
        try priceStore.getPrices(for: assetIds).map { $0.toGem() }
    }

    public func getRate(currency: Gemstone.Currency) async throws -> Gemstone.FiatRate? {
        let currency = currency.toPrimitives()
        return try priceStore.getRate(currency: currency.rawValue).map { Primitives.FiatRate(symbol: currency, rate: $0.rate).toGem() }
    }

    public func getRates() async throws -> [Gemstone.FiatRate] {
        try priceStore.getRates().map { Primitives.FiatRate(symbol: $0.symbol, rate: $0.rate).toGem() }
    }

    public func saveRates(rates: [Gemstone.FiatRate], conversion: Gemstone.FiatRate?) async throws {
        try priceStore.saveRates(rates.map { $0.toPrimitives() }, conversion: conversion?.toPrimitives())
    }

    public func savePrices(currency _: Gemstone.Currency, prices: [GemPriceUpdate]) async throws {
        try priceStore.updatePrices(prices.map { update in
            try PriceUpdate(
                assetId: Primitives.AssetId(id: update.assetId),
                price: update.price,
                priceUsd: update.priceUsd,
                priceChangePercentage24h: update.priceChangePercentage24h,
                updatedAt: update.updatedAt,
            )
        })
    }

    public func convertPrices(currency _: Gemstone.Currency, rate: Double) async throws {
        try priceStore.convertPrices(rate: rate)
    }

    public func saveMarket(assetId: Gemstone.AssetId, market: Gemstone.AssetMarket) async throws {
        try priceStore.updateMarket(assetId: assetId, market: market.toPrimitives())
    }
}
