// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.assetIdsEnabledByDefault
import protocol Gemstone.GemPriceServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct PriceService: Sendable {
    private let priceStore: PriceStore
    private let service: any GemPriceServiceProtocol

    public init(
        priceStore: PriceStore,
        service: any GemPriceServiceProtocol,
    ) {
        self.priceStore = priceStore
        self.service = service
    }

    public func updatePrices(_ prices: [AssetPrice], currency: String) async throws {
        try await service.updatePrices(prices: prices.map { try $0.json() }, currency: currencyJson(currency))
    }

    public func updateAssetPrice(assetId: AssetId, price: AssetPrice?, currency: String) async throws {
        try await service.updateAssetPrice(assetId: assetId.identifier, price: price.map { try $0.json() }, currency: currencyJson(currency))
    }

    public func addRates(_ rates: [FiatRate], currency: String) async throws {
        try await service.updateRates(rates: rates.map { try $0.json() }, currency: currencyJson(currency))
    }

    public func changeCurrency(currency: String) async throws {
        try await service.changeCurrency(currency: currencyJson(currency))
    }

    public func updateMarketPrice(assetId: AssetId, market: AssetMarket, currency: String) async throws {
        try await service.updateMarket(assetId: assetId.identifier, market: market.json(), currency: currencyJson(currency))
    }

    public func getPrice(for assetId: AssetId) throws -> AssetPrice? {
        try priceStore.getPrices(for: [assetId.identifier]).first
    }

    public func getPrices(for assetIds: [AssetId]) throws -> [AssetPrice] {
        try priceStore.getPrices(for: assetIds.map(\.identifier))
    }

    public func observableAssets(walletId: WalletId) throws -> [AssetId] {
        let priceAssets = try priceStore.enabledPriceAssets(walletId: walletId)
        if priceAssets.isEmpty {
            return try assetIdsEnabledByDefault().map(AssetId.init(id:))
        }
        return priceAssets
    }

    public func getRate(currency: String) throws -> Double {
        try priceStore.getRate(currency: currency).rate
    }

    @discardableResult
    public func clear() throws -> Int {
        try priceStore.clear()
    }

    private func currencyJson(_ currency: String) throws -> String {
        guard let currency = Currency(rawValue: currency) else {
            throw AnyError("unknown currency: \(currency)")
        }
        return try currency.json()
    }
}
