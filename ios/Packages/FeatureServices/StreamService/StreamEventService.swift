// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import FiatService
import Foundation
import NFTService
import PerpetualService
import Preferences
import PriceAlertService
import PriceService
import Primitives
import Store
import SupportChatService
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
    private let fiatService: FiatService
    private let supportChatService: SupportChatService
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
        fiatService: FiatService,
        supportChatService: SupportChatService,
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
        self.fiatService = fiatService
        self.supportChatService = supportChatService
        self.preferences = preferences
    }

    public func handle(_ event: StreamEvent) async {
        switch event {
        case let .prices(payload):
            debugLog("stream event handler: prices: \(payload.prices.count), rates: \(payload.rates.count)")
            await perform { try await handlePrices(payload) }
        case let .balances(update):
            debugLog("stream event handler: balances: wallet: \(update.walletId.id), assets: \(update.assetIds.map(\.identifier))")
            Task { await perform { try await handleBalanceUpdate(update) } }
        case let .transactions(update):
            debugLog("stream event handler: transactions: wallet: \(update.walletId.id), transactions: \(update.transactions.map(\.identifier)), assets: \(update.assetIds.map(\.identifier))")
            Task { await perform { try await handleTransactionUpdate(update) } }
        case let .nft(update):
            debugLog("stream event handler: nft: wallet: \(update.walletId.id)")
            Task { await perform { try await handleNftUpdate(update) } }
        case let .perpetual(update):
            debugLog("stream event handler: perpetual: wallet: \(update.walletId.id)")
            Task { await perform { try await handlePerpetualUpdate(update) } }
        case let .inAppNotification(update):
            debugLog("stream event handler: in-app notification: wallet: \(update.walletId.id), id: \(update.notification.item.id)")
            await perform { try notificationStore.addNotifications([update.notification]) }
        case let .priceAlerts(update):
            debugLog("stream event handler: price alerts: assets: \(update.assets.map(\.identifier))")
            Task { await perform { try await priceAlertService.update() } }
        case let .fiatTransaction(update):
            debugLog("stream event handler: fiat transaction: wallet: \(update.walletId.id)")
            Task { await perform { try await handleFiatTransactionUpdate(update) } }
        case let .support(supportEvent):
            switch supportEvent {
            case let .message(message):
                debugLog("stream event handler: support message: id: \(message.id), images: \(message.images.count)")
            case let .typing(typing):
                debugLog("stream event handler: support typing: status: \(typing.status.rawValue)")
            }
            await perform { try await supportChatService.receive(supportEvent) }
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

    private func handlePrices(_ payload: WebSocketPricePayload) async throws {
        try await priceService.addRates(payload.rates, currency: preferences.currency)
        try await priceService.updatePrices(payload.prices, currency: preferences.currency)
    }

    private func handleBalanceUpdate(_ update: StreamBalanceUpdate) async throws {
        guard let wallet = try walletStore.getWallet(id: update.walletId) else { return }
        await balanceUpdater.updateBalance(for: wallet, assetIds: update.assetIds)
    }

    private func handleTransactionUpdate(_ update: StreamTransactionsUpdate) async throws {
        guard let wallet = try walletStore.getWallet(id: update.walletId) else { return }
        try await transactionsService.updateAll(walletId: update.walletId)
        await balanceUpdater.updateBalance(for: wallet, assetIds: update.assetIds)
    }

    private func handleNftUpdate(_ update: StreamWalletUpdate) async throws {
        guard let wallet = try walletStore.getWallet(id: update.walletId) else { return }
        try await nftService.updateAssets(wallet: wallet)
    }

    private func handlePerpetualUpdate(_ update: StreamWalletUpdate) async throws {
        guard let wallet = try walletStore.getWallet(id: update.walletId), let account = wallet.hyperliquidAccount else { return }
        try await perpetualService.getPositions(walletId: update.walletId, address: account.address)
    }

    private func handleFiatTransactionUpdate(_ update: StreamWalletUpdate) async throws {
        try await fiatService.updateTransactions(walletId: update.walletId)
    }
}
