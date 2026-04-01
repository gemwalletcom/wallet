// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Foundation
import NFTService
import PerpetualService
import Preferences
import PriceAlertService
import PriceService
import Primitives
import Store
import TransactionsService

public struct StreamEventService: Sendable {
    private let walletStore: WalletStore
    private let notificationStore: InAppNotificationStore
    private let priceService: PriceService
    private let priceAlertService: PriceAlertService
    private let balanceUpdater: any BalanceUpdater
    private let transactionsService: TransactionsService
    private let nftService: NFTService
    private let perpetualService: any HyperliquidPerpetualServiceable
    private let preferences: Preferences

    public init(
        walletStore: WalletStore,
        notificationStore: InAppNotificationStore,
        priceService: PriceService,
        priceAlertService: PriceAlertService,
        balanceUpdater: any BalanceUpdater,
        transactionsService: TransactionsService,
        nftService: NFTService,
        perpetualService: any HyperliquidPerpetualServiceable,
        preferences: Preferences,
    ) {
        self.walletStore = walletStore
        self.notificationStore = notificationStore
        self.priceService = priceService
        self.priceAlertService = priceAlertService
        self.balanceUpdater = balanceUpdater
        self.transactionsService = transactionsService
        self.nftService = nftService
        self.perpetualService = perpetualService
        self.preferences = preferences
    }

    public func handle(_ event: StreamEvent) async {
        switch event {
        case let .prices(payload):
            await perform { try handlePrices(payload) }
        case let .balances(updates):
            Task { await perform { try await handleBalanceUpdates(updates) } }
        case let .transactions(update):
            Task { await perform { try await transactionsService.updateAll(walletId: update.walletId) } }
        case let .nft(update):
            Task { await perform { try await handleNftUpdate(update) } }
        case let .perpetual(update):
            Task { await perform { try await handlePerpetualUpdate(update) } }
        case let .inAppNotification(update):
            await perform { try notificationStore.addNotifications([update.notification]) }
        case .priceAlerts:
            Task { await perform { try await priceAlertService.update() } }
        }
    }
}

// MARK: - Private

extension StreamEventService {
    private func perform(_ body: () async throws -> Void) async {
        do {
            try await body()
        } catch {
            debugLog("stream event handler error: \(error)")
        }
    }

    private func handlePrices(_ payload: WebSocketPricePayload) throws {
        debugLog("stream event handler: prices: \(payload.prices.count), rates: \(payload.rates.count)")
        try priceService.addRates(payload.rates)
        try priceService.updatePrices(payload.prices, currency: preferences.currency)
    }

    private func handleBalanceUpdates(_ updates: [StreamBalanceUpdate]) async throws {
        for (walletId, walletUpdates) in Dictionary(grouping: updates, by: \.walletId) {
            guard let wallet = try walletStore.getWallet(id: walletId) else { continue }
            await balanceUpdater.updateBalance(for: wallet, assetIds: walletUpdates.map(\.assetId))
        }
    }

    private func handleNftUpdate(_ update: StreamNftUpdate) async throws {
        guard let wallet = try walletStore.getWallet(id: update.walletId) else { return }
        try await nftService.updateAssets(wallet: wallet)
    }

    private func handlePerpetualUpdate(_ update: StreamPerpetualUpdate) async throws {
        guard let wallet = try walletStore.getWallet(id: update.walletId), let account = wallet.hyperliquidAccount else { return }
        try await perpetualService.fetchPositions(walletId: update.walletId, address: account.address)
    }
}
