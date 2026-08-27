// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualSocketUpdate
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives

public struct PerpetualService: PerpetualServiceable {
    private let provider: PerpetualProvidable
    private let service: any GemPerpetualServiceProtocol
    private let appPreferences: Preferences

    public init(
        provider: PerpetualProvidable,
        service: any GemPerpetualServiceProtocol,
        appPreferences: Preferences = .standard,
    ) {
        self.provider = provider
        self.service = service
        self.appPreferences = appPreferences
    }

    public func updateMarkets() async throws {
        try await service.syncMarketsIfStale(chain: Chain.hyperCore.rawValue, currency: Currency(id: appPreferences.currency).json())
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

    @discardableResult
    public func getPositions(walletId: WalletId, address: String) async throws -> PerpetualAccountMode {
        try await PerpetualAccountMode(service.syncPositions(walletId: walletId.id, chain: Chain.hyperCore.rawValue, address: address))
    }
}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualService: HyperliquidPerpetualServiceable {
    public func accountMode(walletId: WalletId, address: String) async -> PerpetualAccountMode {
        do {
            return try await PerpetualAccountMode(service.accountMode(walletId: walletId.id, chain: Chain.hyperCore.rawValue, address: address))
        } catch {
            debugLog("PerpetualService: account mode failed: \(error)")
            return .standard
        }
    }

    public func applySocketMessage(walletId: WalletId, mode: PerpetualAccountMode, data: Data) async throws -> GemPerpetualSocketUpdate {
        try await service.applySocketMessage(walletId: walletId.id, mode: mode.json(), data: data)
    }
}
