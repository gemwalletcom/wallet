// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Components
import ConnectionsService
import EventPresenterService
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI
import TransactionsService
import WalletConnector
import WalletSessionService

@Observable
final class NavigationHandler: Sendable {
    private let navigationState: NavigationStateManager
    private let presenter: NavigationPresenter

    private let assetsService: AssetsService
    private let connectionsService: ConnectionsService
    private let eventPresenterService: EventPresenterService
    private let transactionsService: TransactionsService
    private let walletConnectorPresenter: WalletConnectorPresenter
    private let walletSessionService: any WalletSessionManageable

    init(
        navigationState: NavigationStateManager,
        presenter: NavigationPresenter,
        assetsService: AssetsService,
        connectionsService: ConnectionsService,
        eventPresenterService: EventPresenterService,
        transactionsService: TransactionsService,
        walletConnectorPresenter: WalletConnectorPresenter,
        walletSessionService: any WalletSessionManageable,
    ) {
        self.navigationState = navigationState
        self.presenter = presenter
        self.assetsService = assetsService
        self.connectionsService = connectionsService
        self.eventPresenterService = eventPresenterService
        self.transactionsService = transactionsService
        self.walletConnectorPresenter = walletConnectorPresenter
        self.walletSessionService = walletSessionService
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
    func handle(url: URL) async {
        await handle(code: url.absoluteString)
    }

    @MainActor
    func handle(code: String) async {
        guard let action = try? URLParser.from(code: code) else {
            return presentNotSupported()
        }
        await handle(action)
    }

    @MainActor
    func handle(_ action: URLAction) async {
        do {
            try await handleURLAction(action)
        } catch {
            debugLog("NavigationHandler URLAction error: \(error)")
            presentNotSupported()
        }
    }

    @MainActor
    func open(url: URL) -> Bool {
        guard let action = try? URLParser.from(url: url) else { return false }
        Task { await handle(action) }
        return true
    }
}

// MARK: - URLAction

@MainActor
extension NavigationHandler {
    private func handleURLAction(_ action: URLAction) async throws {
        switch action {
        case let .deeplink(deeplink): try await handleDeepLink(deeplink)
        case let .payment(payment): try handlePayment(payment)
        case let .walletConnect(action): await handleWalletConnect(action)
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

// MARK: - Payment

@MainActor
extension NavigationHandler {
    private func handlePayment(_ payment: Payment) throws {
        guard case let .request(request) = payment, let wallet = walletSessionService.currentWallet else {
            throw AnyError(Localized.Errors.notSupported)
        }
        let assets = try assetsService.assetStore.getAssetsData(walletId: wallet.id, filters: [.enabledBalance])

        presenter.isPresentingPayment.wrappedValue = try PaymentInputBuilder.build(payment: request, assets: assets)
    }
}

// MARK: - WalletConnect

@MainActor
extension NavigationHandler {
    private func handleWalletConnect(_ action: WalletConnectAction) async {
        walletConnectorPresenter.isPresentingConnectionBar = true

        do {
            switch action {
            case let .connect(uri):
                try await connectionsService.pair(uri: uri)
            case .request:
                break
            case .session:
                connectionsService.updateSessions()
            }
        } catch {
            debugLog("NavigationHandler walletConnect error: \(error)")
            walletConnectorPresenter.isPresentingError = error.localizedDescription
        }
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
    private func presentNotSupported() {
        eventPresenterService.toastPresenter.toastMessage = .error(Localized.Errors.notSupported)
    }

    private func selectTab(for tab: TabItem?) {
        guard let tab else { return }
        navigationState.selectedTab = tab
    }

    private func navigateToAsset(_ assetId: AssetId) async throws {
        guard let asset = try await preparedAssetForNavigation(assetId: assetId, wallet: walletSessionService.currentWallet) else {
            return
        }
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
        guard let wallet = try? walletSessionService.getWallet(walletId: walletId) else {
            return nil
        }
        return try await preparedAssetForNavigation(assetId: assetId, wallet: wallet)
    }

    private func preparedAssetForNavigation(assetId: AssetId, wallet: Wallet?) async throws -> Asset? {
        guard let wallet, wallet.accounts.contains(where: { $0.chain == assetId.chain }) else {
            return nil
        }
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try assetsService.addBalancesIfMissing(walletId: wallet.id, assetIds: [asset.id])
        return asset
    }

    private func selectWalletIfNeeded(_ walletId: WalletId) async {
        guard walletSessionService.currentWalletId != walletId else {
            return
        }

        walletSessionService.setCurrent(walletId: walletId)
        await withCheckedContinuation { continuation in
            RunLoop.main.perform(inModes: [.common]) {
                continuation.resume()
            }
        }
    }

    private func presentSwap(from fromId: AssetId, to toId: AssetId?) async throws {
        guard let wallet = walletSessionService.currentWallet else { return }
        try await presenter.presentSwap(from: fromId, to: toId, wallet: wallet, assetsService: assetsService)
    }

    private func presentBuy(assetId: AssetId, amount: Int?) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try presentAssetInput(type: .buy(asset, amount: amount), for: asset)
    }

    private func presentAssetInput(type: SelectedAssetType, for asset: Asset) throws {
        guard let wallet = walletSessionService.currentWallet else { return }
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
