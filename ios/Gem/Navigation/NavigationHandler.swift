// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import Primitives
import SwiftUI
import TransactionsService
import WalletService

@Observable
final class NavigationHandler: Sendable {
    private let navigationState: NavigationStateManager
    private let presenter: NavigationPresenter

    private let assetsService: AssetsService
    private let transactionsService: TransactionsService
    private let walletService: WalletService

    @MainActor var wallet: Wallet?

    init(
        navigationState: NavigationStateManager,
        presenter: NavigationPresenter,
        assetsService: AssetsService,
        transactionsService: TransactionsService,
        walletService: WalletService,
    ) {
        self.navigationState = navigationState
        self.presenter = presenter
        self.assetsService = assetsService
        self.transactionsService = transactionsService
        self.walletService = walletService
    }

    @MainActor
    func handlePush(_ userInfo: [AnyHashable: Any]) async {
        do {
            let notification = try PushNotification(from: userInfo)
            try await handle(notification)
        } catch {
            debugLog("NavigationHandler push error: \(error)")
        }
    }

    @MainActor
    func handle(_ action: URLAction) async {
        do {
            try await handleURLAction(action)
        } catch {
            debugLog("NavigationHandler URLAction error: \(error)")
        }
    }
}

// MARK: - URLAction

@MainActor
extension NavigationHandler {
    private func handleURLAction(_ action: URLAction) async throws {
        switch action {
        case .walletConnect: break
        case let .deeplink(deeplink): try await handleDeepLink(deeplink)
        }
    }

    private func handleDeepLink(_ deeplink: DeepLink) async throws {
        switch deeplink {
        case let .asset(assetId):
            try await navigateToAsset(assetId)

        case .perpetuals:
            navigationState.wallet.append(Scenes.Perpetuals())

        case let .rewards(code):
            navigationState.settings.append(Scenes.Referral(code: code))
        }

        selectTab(for: deeplink.selectTab)
    }
}

// MARK: - PushNotification

@MainActor
extension NavigationHandler {
    private func handle(_ notification: PushNotification) async throws {
        switch notification {
        case let .asset(assetId):
            try await navigateToAsset(assetId)
        case let .walletAsset(walletId, assetId):
            try await navigateToAsset(walletId: walletId, assetId: assetId)
        case let .transaction(walletId, assetId, transaction):
            try await navigateToTransaction(walletId: walletId, assetId: assetId, transaction: transaction)
        case let .priceAlert(assetId):
            try await navigateToAsset(assetId)
        case let .buyAsset(assetId, amount):
            try await presentBuy(assetId: assetId, amount: amount)
        case let .swapAsset(fromId, toId):
            try await presentSwap(from: fromId, to: toId)
        case .support:
            presenter.isPresentingSupport.wrappedValue = true
        case .rewards:
            navigationState.settings.append(Scenes.Referral(code: nil))
        case .stake: break
        // TODO: Select wallet and open stake screen of an asset
        case .test, .unknown: break
        }

        selectTab(for: notification.selectTab)
    }
}

// MARK: - Private

@MainActor
extension NavigationHandler {
    private func selectTab(for tab: TabItem?) {
        guard let tab else { return }
        navigationState.selectedTab = tab
    }

    private func navigateToAsset(_ assetId: AssetId) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        navigationState.openAsset(asset)
    }

    private func navigateToAsset(walletId: WalletId, assetId: AssetId) async throws {
        guard let asset = try await assetForWalletNavigation(walletId: walletId, assetId: assetId) else {
            return
        }

        await selectWalletIfNeeded(walletId)
        navigationState.openAsset(asset)
    }

    private func navigateToTransaction(walletId: WalletId, assetId: AssetId, transaction: Primitives.Transaction) async throws {
        guard let asset = try await assetForWalletNavigation(walletId: walletId, assetId: assetId) else {
            return
        }

        try transactionsService.addTransaction(walletId: walletId, transaction: transaction)
        let transaction = try transactionsService.getTransaction(walletId: walletId, transactionId: transaction.id)

        await selectWalletIfNeeded(walletId)
        switch asset.type {
        case .perpetual:
            navigationState.wallet.setPath([Scenes.Perpetuals(), Scenes.Perpetual(asset), Scenes.Transaction(transaction: transaction)])
        default:
            navigationState.wallet.setPath([Scenes.Asset(asset: asset), Scenes.Transaction(transaction: transaction)])
        }

        navigationState.selectedTab = .wallet
    }

    private func assetForWalletNavigation(walletId: WalletId, assetId: AssetId) async throws -> Asset? {
        guard let _ = try? walletService.getWallet(walletId: walletId) else {
            return nil
        }

        return try await assetsService.getOrFetchAsset(for: assetId)
    }

    private func selectWalletIfNeeded(_ walletId: WalletId) async {
        guard walletService.currentWalletId != walletId else {
            return
        }

        walletService.setCurrent(for: walletId)
        await Task.yield()
    }

    private func presentSwap(from fromId: AssetId, to toId: AssetId?) async throws {
        guard let wallet else { return }
        try await presenter.presentSwap(from: fromId, to: toId, wallet: wallet, assetsService: assetsService)
    }

    private func presentBuy(assetId: AssetId, amount: Int?) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try presentAssetInput(type: .buy(asset, amount: amount), for: asset)
    }

    private func presentAssetInput(type: SelectedAssetType, for asset: Asset) throws {
        guard let wallet else { return }
        try presenter.presentAssetInput(type: type, for: asset, wallet: wallet)
    }

    func resetNavigation() {
        navigationState.clearAll()
        navigationState.selectedTab = .wallet
    }
}

// MARK: - TabItem Selection

private extension DeepLink {
    var selectTab: TabItem? {
        switch self {
        case .asset, .perpetuals: .wallet
        case .rewards: .settings
        }
    }
}

private extension PushNotification {
    var selectTab: TabItem? {
        switch self {
        case .transaction, .asset, .walletAsset, .priceAlert, .stake: .wallet
        case .buyAsset, .swapAsset: nil
        case .support, .rewards: .settings
        case .test, .unknown: nil
        }
    }
}
