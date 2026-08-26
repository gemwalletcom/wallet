// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives
import Store

public struct PerpetualService: PerpetualServiceable {
    private let store: PerpetualStore
    private let perpetualStore: GemstonePerpetualStore
    private let balanceStore: BalanceStore
    private let provider: PerpetualProvidable
    private let service: any GemPerpetualServiceProtocol
    private let preferences: Preferences

    public init(
        store: PerpetualStore,
        perpetualStore: GemstonePerpetualStore,
        balanceStore: BalanceStore,
        provider: PerpetualProvidable,
        service: any GemPerpetualServiceProtocol,
        preferences: Preferences,
    ) {
        self.store = store
        self.perpetualStore = perpetualStore
        self.balanceStore = balanceStore
        self.provider = provider
        self.service = service
        self.preferences = preferences
    }

    public var marketsUpdatedAt: Date? {
        preferences.perpetualMarketsUpdatedAt
    }

    public func updateMarkets() async throws {
        try await service.syncMarkets(chain: Chain.hyperCore.rawValue)
        preferences.perpetualMarketsUpdatedAt = .now
    }

    public func clearMarkets() throws {
        try clear()
        try clearBalance()
        preferences.perpetualMarketsUpdatedAt = nil
    }

    public func candlesticks(symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick] {
        try await provider.getCandlesticks(symbol: symbol, period: period)
    }

    public func portfolio(address: String) async throws -> PerpetualPortfolio {
        try await provider.getPortfolio(address: address)
    }

    public func setPinned(_ isPinned: Bool, perpetualId: PerpetualId) throws {
        try store.setPinned(for: [perpetualId.identifier], value: isPinned)
    }

    public func getPositions(walletId: WalletId, address: String) async throws {
        try await service.syncPositions(walletId: walletId.id, chain: Chain.hyperCore.rawValue, address: address)
    }

    // MARK: - Private

    private func clear() throws {
        try store.clear()
    }

    private func clearBalance() throws {
        try balanceStore.deleteBalance(assetId: Chain.hyperCore.defaultAsset(type: .perpetual).id)
    }

}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualService: HyperliquidPerpetualServiceable {
    public func accountMode(walletId: WalletId, address: String) async -> PerpetualAccountMode {
        let walletPreferences = WalletPreferences(walletId: walletId)
        do {
            let mode = try await provider.getAccountMode(address: address)
            walletPreferences.perpetualAccountMode = mode
            return mode
        } catch {
            debugLog("PerpetualService: account mode failed: \(error)")
            return walletPreferences.perpetualAccountMode
        }
    }

    public func getHypercorePositions(walletId: WalletId) throws -> [Primitives.PerpetualPosition] {
        try store.getPositions(walletId: walletId, provider: .hypercore)
    }

    public func updateBalance(walletId: WalletId, balance: Primitives.PerpetualBalance) throws {
        try perpetualStore.updateBalance(walletId: walletId, balance: balance)
    }

    public func diffPositions(deleteIds: [String], positions: [Primitives.PerpetualPosition], walletId: WalletId) throws {
        try store.diffPositions(deleteIds: deleteIds, positions: positions, walletId: walletId)
    }

    public func updateMarket(_ market: Primitives.PerpetualMarketData) throws {
        try store.updateMarket(
            coin: market.coin,
            price: market.price,
            pricePercentChange24h: market.pricePercentChange24h,
            openInterest: market.openInterest,
            volume24h: market.volume24h,
            funding: market.funding,
        )
    }

    public func updatePrices(_ prices: [String: Double]) throws {
        guard preferences.perpetualPricesUpdatedAt.isOutdated(by: PerpetualConfig.pricesUpdateIntervalSeconds) else { return }
        try store.updatePrices(prices)
        preferences.perpetualPricesUpdatedAt = .now
    }
}
