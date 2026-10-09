// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import enum Gemstone.GemNameInputStep
import struct Gemstone.GemNumberFormat
import struct Gemstone.GemPriceAlertSession
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemPriceAlertServiceMock: GemPriceAlertServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var enabled: Bool
    private let setEnabledError: Error?
    private let allowsNotifications: Bool
    public var refreshState: GemLoadState = .data

    public init(enabled: Bool = false, setEnabledError: Error? = .none, allowsNotifications: Bool = true) {
        self.enabled = enabled
        self.setEnabledError = setEnabledError
        self.allowsNotifications = allowsNotifications
    }

    public func newAlertSession(assetId: Gemstone.AssetId, format: GemNumberFormat) -> GemPriceAlertSession {
        GemPriceAlertSession(
            assetId: assetId,
            currency: getCurrency(),
            notificationType: .price,
            selectedDirection: .up,
            input: nil,
            currentPrice: nil,
            priceChange: nil,
            isSaving: false,
            format: format,
        )
    }

    public func isEnabled() -> Bool {
        lock.withLock { enabled }
    }

    public func setEnabled(enabled: Bool) async throws {
        if let setEnabledError {
            throw setEnabledError
        }
        lock.withLock { self.enabled = enabled }
    }

    public func refresh(assetId _: Gemstone.AssetId?) async -> GemLoadState {
        refreshState
    }

    public func enablePriceAlert(alert _: Gemstone.PriceAlert) async throws -> Bool {
        if allowsNotifications {
            lock.withLock { enabled = true }
        }
        return allowsNotifications
    }

    public func deletePriceAlerts(alerts _: [Gemstone.PriceAlert]) async throws {}

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func setAutoAlert(asset: Gemstone.Asset, enabled isEnabled: Bool) async throws -> GemToast? {
        if isEnabled, !allowsNotifications {
            return nil
        }
        lock.withLock { enabled = isEnabled }
        return GemToast(text: .priceAlertsToggled(name: asset.name, enabled: isEnabled), icon: .priceAlert)
    }

    public func priceAlertId(alert: Gemstone.PriceAlert) -> String {
        alert.toPrimitives().id
    }
}

public final class GemPerpetualServiceMock: GemPerpetualServiceProtocol, @unchecked Sendable {
    public var isAvailableValue = true
    public private(set) var availabilityCheckCount = 0

    public func isAvailable() -> Bool {
        availabilityCheckCount += 1
        return isAvailableValue
    }

    public var isPerpetualEnabled = true
    public var connects = true
    public var enablementError: Error?
    public private(set) var syncMarketsCount = 0
    public private(set) var syncPositionsCount = 0
    public private(set) var clearMarketsCount = 0
    public var connectionFailures = 0
    public var connectionGate: (@Sendable (Gemstone.Wallet) async -> Void)?
    public private(set) var connectionCount = 0
    public var depositTargetValue: Gemstone.GemPerpetualDepositTarget = .selectAsset
    private var updatedAt: Int64?

    public init(marketsUpdatedAt: Int64? = nil) {
        updatedAt = marketsUpdatedAt
    }

    public func marketsUpdatedAt() throws -> Int64? {
        updatedAt
    }

    public func syncEnablement(wallet _: Gemstone.Wallet?) async throws -> Bool {
        if let enablementError {
            throw enablementError
        }
        guard isPerpetualEnabled else {
            try await clearMarkets()
            return false
        }
        if updatedAt == nil {
            try await syncMarkets(chain: "hypercore")
        }
        return connects
    }

    private func syncMarketsIfNeeded(chain: Gemstone.Chain, trigger: Gemstone.GemMarketsRefreshTrigger) async throws -> Bool {
        if trigger == .scheduled, updatedAt != nil {
            return false
        }
        try await syncMarkets(chain: chain)
        return true
    }

    private func syncMarkets(chain _: Gemstone.Chain) async throws {
        syncMarketsCount += 1
        updatedAt = Int64(Date().timeIntervalSince1970)
    }

    public func clearMarkets() async throws {
        clearMarketsCount += 1
        updatedAt = nil
    }

    private func syncCurrentPositions() async throws {
        syncPositionsCount += 1
    }

    public func depositTarget() async throws -> Gemstone.GemPerpetualDepositTarget {
        depositTargetValue
    }

    public func refresh(trigger: Gemstone.GemMarketsRefreshTrigger) async -> [Gemstone.GemPerpetualRefreshFailure] {
        try? await syncCurrentPositions()
        _ = try? await syncMarketsIfNeeded(chain: "hypercore", trigger: trigger)
        return []
    }

    public func connection(wallet: Gemstone.Wallet) async throws -> Gemstone.GemPerpetualConnection? {
        connectionCount += 1
        await connectionGate?(wallet)
        if connectionFailures > 0 {
            connectionFailures -= 1
            throw AnyError("connection unavailable")
        }
        guard let account = wallet.toPrimitives().hyperliquidAccount else { return nil }
        return Gemstone.GemPerpetualConnection(
            address: account.address,
            mode: Primitives.PerpetualAccountMode.standard.toGem(),
        )
    }

    public func setPinned(perpetualId _: PerpetualId, name: String, pinned: Bool) async throws -> GemToast {
        GemToast(text: .pinned(name: name, pinned: pinned), icon: pinned ? .pin : .unpin)
    }

    public func addRecent(action _: Gemstone.GemAssetAction, asset _: Gemstone.Asset) async throws {}
}

public final class GemPerpetualDetailsServiceMock: GemPerpetualDetailsServiceProtocol, @unchecked Sendable {
    public var isAvailableValue = true

    public func isAvailable() -> Bool {
        isAvailableValue
    }

    public var detailsValue: GemPerpetualDetails = .mock()
    public var chartPeriodValue: Gemstone.ChartPeriod = Primitives.ChartPeriod.day.toGem()
    public var candlesticksValue: [Gemstone.ChartCandleStick] = []
    public var candlesticksError: GemServiceError?
    public var closeTransferResult: Result<Gemstone.GemTransferData, Error> = .success(.mock())
    public var positionActionResult: Result<GemPerpetualPositionAction, Error> = .success(.open(data: .mock(direction: .long, price: 100, leverage: 3)))
    public var syncPositionsError: Error?
    public var syncTransactionsError: Error?

    public private(set) var syncPositionsCount = 0
    public private(set) var syncedTransactionAssetIds: [Gemstone.AssetId] = []
    public private(set) var positionKinds: [GemPerpetualPositionKind] = []

    public init() {}

    public func candleSubscription(perpetual: Gemstone.Perpetual, period: Gemstone.ChartPeriod) -> GemPerpetualSubscription {
        .candle(symbol: perpetual.name, interval: period.toPrimitives().rawValue)
    }

    public func candles(request: GemCandleRequest) async -> GemCandleResult {
        if let candlesticksError {
            return GemCandleResult(request: request, state: .error(error: candlesticksError), candles: [])
        }
        return GemCandleResult(request: request, state: .data, candles: candlesticksValue)
    }

    public func chartPeriod() -> Gemstone.ChartPeriod {
        chartPeriodValue
    }

    public func closeTransfer(perpetual _: Gemstone.Perpetual, asset _: Gemstone.Asset, position _: Gemstone.PerpetualPosition?) throws -> Gemstone.GemTransferData {
        try closeTransferResult.get()
    }

    public func marketSubscription(perpetual: Gemstone.Perpetual) -> GemPerpetualSubscription {
        .marketData(symbol: perpetual.name)
    }

    public func positionAction(
        perpetual _: Gemstone.Perpetual,
        asset _: Gemstone.Asset,
        position _: Gemstone.PerpetualPosition?,
        kind: GemPerpetualPositionKind,
    ) throws -> GemPerpetualPositionAction {
        positionKinds.append(kind)
        return try positionActionResult.get()
    }

    public func details(perpetual _: Gemstone.Perpetual, asset _: Gemstone.Asset, positions _: [Gemstone.PerpetualPosition]) -> GemPerpetualDetails {
        detailsValue
    }

    public func refresh(assetId: Gemstone.AssetId) async -> [Gemstone.GemPerpetualRefreshFailure] {
        syncPositionsCount += 1
        syncedTransactionAssetIds.append(assetId)
        return [
            syncPositionsError.map { GemPerpetualRefreshFailure(step: .positions, message: $0.localizedDescription) },
            syncTransactionsError.map { GemPerpetualRefreshFailure(step: .transactions, message: $0.localizedDescription) },
        ].compactMap(\.self)
    }
}

public final class GemPerpetualStreamServiceMock: GemPerpetualStreamServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var connectedAddresses: [String] = []

    public init() {}

    public var addresses: [String] { lock.withLock { connectedAddresses } }

    public func connected(address: String, mode _: Gemstone.PerpetualAccountMode) async throws {
        lock.withLock { connectedAddresses.append(address) }
    }

    public func disconnected() async {}

    public func candleUpdate(walletId _: Gemstone.WalletId, mode _: Gemstone.PerpetualAccountMode, data _: Data) async throws -> Gemstone.ChartCandleUpdate? {
        nil
    }

    public func subscribe(subscription _: Gemstone.GemPerpetualSubscription) async throws {}
    public func unsubscribe(subscription _: Gemstone.GemPerpetualSubscription) async throws {}
}
