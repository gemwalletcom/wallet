// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletPreferencesServiceProtocol
import Foundation
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives

public struct PerpetualService: PerpetualServiceable {
    private let provider: PerpetualProvidable
    private let service: any GemPerpetualServiceProtocol
    private let preferences: any GemWalletPreferencesServiceProtocol
    private let appPreferences: Preferences

    public init(
        provider: PerpetualProvidable,
        service: any GemPerpetualServiceProtocol,
        preferences: any GemWalletPreferencesServiceProtocol,
        appPreferences: Preferences = .standard,
    ) {
        self.provider = provider
        self.service = service
        self.preferences = preferences
        self.appPreferences = appPreferences
    }

    public var marketsUpdatedAt: Date? {
        ((try? service.marketsUpdatedAt()) ?? nil).map { Date(timeIntervalSince1970: TimeInterval($0)) }
    }

    public func updateMarkets() async throws {
        try await service.syncMarkets(chain: Chain.hyperCore.rawValue, currency: Currency(id: appPreferences.currency).json())
    }

    public func clearMarkets() async throws {
        try await service.clearMarkets()
    }

    public func candlesticks(symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick] {
        try await provider.getCandlesticks(symbol: symbol, period: period)
    }

    public func portfolio(address: String) async throws -> PerpetualPortfolio {
        try await provider.getPortfolio(address: address)
    }

    public func setPinned(_ isPinned: Bool, perpetualId: PerpetualId) async throws {
        try await service.setPinned(perpetualId: perpetualId.identifier, pinned: isPinned)
    }

    public func getPositions(walletId: WalletId, address: String) async throws {
        try await service.syncPositions(walletId: walletId.id, chain: Chain.hyperCore.rawValue, address: address)
    }
}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualService: HyperliquidPerpetualServiceable {
    public func accountMode(walletId: WalletId, address: String) async -> PerpetualAccountMode {
        do {
            let mode = try await provider.getAccountMode(address: address)
            try preferences.setPerpetualAccountMode(walletId: walletId, mode: mode)
            return mode
        } catch {
            debugLog("PerpetualService: account mode failed: \(error)")
            return (try? preferences.getPerpetualAccountMode(walletId: walletId)) ?? .standard
        }
    }

    public func getHypercorePositions(walletId: WalletId) async throws -> [Primitives.PerpetualPosition] {
        try await service.getPositions(walletId: walletId.id, chain: Chain.hyperCore.rawValue).map { try Primitives.PerpetualPosition($0) }
    }

    public func updateBalance(walletId: WalletId, balance: Primitives.PerpetualBalance) async throws {
        try await service.updateBalance(walletId: walletId.id, balance: balance.json())
    }

    public func updatePositions(walletId: WalletId, positions: [Primitives.PerpetualPosition], deleteIds: [String]) async throws {
        try await service.updatePositions(walletId: walletId.id, positions: positions.map { try $0.json() }, deleteIds: deleteIds)
    }

    public func updateMarket(_ market: Primitives.PerpetualMarketData) async throws {
        try await service.updateMarket(market: market.json())
    }

    public func updatePrices(_ prices: [String: Double]) async throws {
        try await service.updatePrices(prices: prices)
    }
}
