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

    public func saveRatesAndPrices(currency _: Gemstone.Currency, rates: [Gemstone.FiatRate], conversion: Gemstone.FiatRate?, prices: [GemPriceUpdate]) async throws {
        try priceStore.saveRatesAndPrices(rates.map { $0.toPrimitives() }, conversion: conversion?.toPrimitives(), prices: prices.map { try $0.priceUpdate() })
    }

    public func savePrices(currency _: Gemstone.Currency, prices: [GemPriceUpdate]) async throws {
        try priceStore.updatePrices(prices.map { try $0.priceUpdate() })
    }

    public func convertPrices(currency _: Gemstone.Currency, rate: Double) async throws {
        try priceStore.convertPrices(rate: rate)
    }

    public func saveMarket(assetId: Gemstone.AssetId, market: Gemstone.AssetMarket) async throws {
        try priceStore.updateMarket(assetId: Primitives.AssetId(id: assetId), market: market.toPrimitives())
    }
}

private extension GemPriceUpdate {
    func priceUpdate() throws -> PriceUpdate {
        try PriceUpdate(
            assetId: Primitives.AssetId(id: assetId),
            price: price,
            priceUsd: priceUsd,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: updatedAt,
        )
    }
}
