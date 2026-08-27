// Copyright (c). Gem Wallet. All rights reserved.

import Store
import GemstoneServices
import Components
import ConnectionsService
import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import class Gemstone.PaymentService
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import WalletConnector

@Observable
final class NavigationHandler: Sendable {
    private let navigationState: NavigationStateManager
    private let presenter: NavigationPresenter

    private let assetsService: any GemAssetsServiceProtocol
    private let assetStore: AssetStore
    private let connectionsService: ConnectionsService
    private let toastPresenter: ToastPresenter
    private let paymentService: PaymentService
    private let transactionStore: TransactionStore
    private let walletConnectorPresenter: WalletConnectorPresenter
    private let walletSessionService: any WalletSessionManageable

    init(
        navigationState: NavigationStateManager,
        presenter: NavigationPresenter,
        assetsService: any GemAssetsServiceProtocol,
        assetStore: AssetStore,
        connectionsService: ConnectionsService,
        toastPresenter: ToastPresenter,
        paymentService: PaymentService,
        transactionStore: TransactionStore,
        walletConnectorPresenter: WalletConnectorPresenter,
        walletSessionService: any WalletSessionManageable,
    ) {
        self.navigationState = navigationState
        self.presenter = presenter
        self.assetsService = assetsService
        self.assetStore = assetStore
        self.connectionsService = connectionsService
        self.toastPresenter = toastPresenter
        self.paymentService = paymentService
        self.transactionStore = transactionStore
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
            return showError(AnyError(Localized.Errors.notSupported))
        }
        await handle(action)
    }

    @MainActor
    func handle(_ action: URLAction) async {
        do {
            try await handleURLAction(action)
        } catch {
            toastPresenter.toastMessage = nil
            showError(error)
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
        case let .payment(payment): try await handlePayment(payment)
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

        case let .receive(assetId):
            try await presentReceive(assetId: assetId)

        case let .buy(assetId, amount):
            try await presentFiat(type: .buy, assetId: assetId, amount: amount)

        case let .sell(assetId, amount):
            try await presentFiat(type: .sell, assetId: assetId, amount: amount)

        case let .swap(assetId):
            try await presentSwap(from: assetId, to: .none)
        }

        selectTab(for: deeplink.selectTab)
    }
}

// MARK: - Payment

@MainActor
extension NavigationHandler {
    private func handlePayment(_ payment: Payment) async throws {
        guard let wallet = walletSessionService.currentWallet else { return }
        switch payment {
        case let .request(request):
            let assets = try assetStore.getAssetsData(walletId: wallet.id, filters: [])
            presenter.isPresentingPayment.wrappedValue = try PaymentDestinationBuilder.build(payment: request, assets: assets)
        case let .link(link):
            toastPresenter.toastMessage = ToastMessage(title: Localized.Common.loading, image: SystemImage.network)
            let addresses = wallet.accounts.map { ChainAddress(chain: $0.chain, address: $0.address) }
            let transaction = try await paymentService.load(link: link, addresses: addresses)
            let chain = try Primitives.ChainAddress(transaction.account).chain
            let assetId = try transaction.request?.map().assetId ?? chain.asset.id
            let asset = try await assetsService.getOrFetchTokenAsset(for: assetId)
            toastPresenter.toastMessage = nil
            presenter.isPresentingPayment.wrappedValue = try PaymentDestinationBuilder.build(transaction: transaction, asset: asset)
        }
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
            try await presentFiat(type: .buy, assetId: assetId, amount: amount)
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
    private func showError(_ error: any Error) {
        debugLog("NavigationHandler error: \(error)")
        toastPresenter.toastMessage = .error(error.localizedDescription)
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

        try transactionStore.addTransactions(walletId: walletId, transactions: [transaction])
        let transaction = try transactionStore.getTransaction(walletId: walletId, transactionId: transaction.id)

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
        guard AssetNavigationPolicy.canOpen(assetId),
              let wallet,
              wallet.accounts.contains(where: { $0.chain == assetId.chain })
        else {
            return nil
        }
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try await assetsService.addMissingBalances(walletId: wallet.id, assetIds: [asset.id])
        return asset
    }

    private func selectWalletIfNeeded(_ walletId: WalletId) async {
        guard walletSessionService.currentWalletId != walletId else {
            return
        }

        do {
            try walletSessionService.setCurrent(walletId: walletId)
        } catch {
            debugLog("set current wallet error: \(error)")
            return
        }
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

    private func presentFiat(type: FiatQuoteType, assetId: AssetId, amount: Int?) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        let selectedType: SelectedAssetType = switch type {
        case .buy: .buy(asset, amount: amount)
        case .sell: .sell(asset, amount: amount)
        }
        try presentAssetInput(type: selectedType, for: asset)
    }

    private func presentReceive(assetId: AssetId) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try presentAssetInput(type: .receive(.asset), for: asset)
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
        case .receive, .buy, .sell, .swap: nil
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
