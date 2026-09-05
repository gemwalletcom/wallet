// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.Deeplink
import protocol Gemstone.GemAssetsServiceProtocol
import class Gemstone.GemDeeplinkService
import class Gemstone.GemPaymentService
import enum Gemstone.GemPushNotification
import protocol Gemstone.GemPushNotificationServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import protocol Gemstone.GemWalletSessionServiceProtocol
import enum Gemstone.UrlAction
import enum Gemstone.WalletConnectLink
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import WalletConnector
import WalletConnectorService

@Observable
final class NavigationHandler: Sendable {
    private let navigationState: NavigationStateManager
    private let presenter: NavigationPresenter

    private let assetsService: any GemAssetsServiceProtocol
    private let assetStore: AssetStore
    private let walletConnector: any WalletConnectorServiceable
    private let toastPresenter: ToastPresenter
    private let pushNotificationService: any GemPushNotificationServiceProtocol
    private let transactionStore: TransactionStore
    private let deeplinkService: GemDeeplinkService
    private let paymentService: GemPaymentService
    private let transactionStateService: any GemTransactionStateServiceProtocol
    private let walletConnectorPresenter: WalletConnectorPresenter
    private let walletSessionService: any GemWalletSessionServiceProtocol

    init(
        navigationState: NavigationStateManager,
        presenter: NavigationPresenter,
        assetsService: any GemAssetsServiceProtocol,
        assetStore: AssetStore,
        walletConnector: any WalletConnectorServiceable,
        toastPresenter: ToastPresenter,
        pushNotificationService: any GemPushNotificationServiceProtocol,
        transactionStore: TransactionStore,
        deeplinkService: GemDeeplinkService,
        paymentService: GemPaymentService,
        transactionStateService: any GemTransactionStateServiceProtocol,
        walletConnectorPresenter: WalletConnectorPresenter,
        walletSessionService: any GemWalletSessionServiceProtocol,
    ) {
        self.navigationState = navigationState
        self.presenter = presenter
        self.assetsService = assetsService
        self.assetStore = assetStore
        self.walletConnector = walletConnector
        self.toastPresenter = toastPresenter
        self.pushNotificationService = pushNotificationService
        self.transactionStore = transactionStore
        self.deeplinkService = deeplinkService
        self.paymentService = paymentService
        self.transactionStateService = transactionStateService
        self.walletConnectorPresenter = walletConnectorPresenter
        self.walletSessionService = walletSessionService
    }

    @MainActor
    func handlePush(_ userInfo: [AnyHashable: Any]) async {
        guard
            let notificationType = userInfo["type"] as? String,
            let notification = pushNotificationService.parse(
                notificationType: notificationType,
                data: Self.payload(userInfo["data"]),
            )
        else {
            return
        }
        do {
            try await handle(notification)
        } catch {
            debugLog("NavigationHandler push error: \(error)")
        }
    }

    private static func payload(_ data: Any?) -> String? {
        guard let data, JSONSerialization.isValidJSONObject(data) else { return .none }
        return (try? JSONSerialization.data(withJSONObject: data)).map { String(decoding: $0, as: UTF8.self) }
    }

    @MainActor
    func handle(url: URL) async {
        await handle(code: url.absoluteString)
    }

    @MainActor
    func handle(code: String) async {
        guard let action = deeplinkService.urlAction(url: code) else {
            return showError(AnyError(Localized.Errors.notSupported))
        }
        await handle(action)
    }

    @MainActor
    func handle(_ action: UrlAction) async {
        do {
            try await handleURLAction(action)
        } catch {
            toastPresenter.toastMessage = nil
            showError(error)
        }
    }

    @MainActor
    func open(url: URL) -> Bool {
        guard let action = deeplinkService.urlAction(url: url.absoluteString) else { return false }
        Task { await handle(action) }
        return true
    }
}

// MARK: - UrlAction

@MainActor
extension NavigationHandler {
    private func handleURLAction(_ action: UrlAction) async throws {
        switch action {
        case let .deeplink(deeplink): try await handleDeepLink(deeplink)
        case let .payment(payment): try await handlePayment(Payment(payment))
        case let .walletConnect(link): await handleWalletConnect(link)
        }
    }

    private func handleDeepLink(_ deeplink: Deeplink) async throws {
        switch deeplink {
        case let .asset(assetId):
            try await navigateToAsset(AssetId(id: assetId))

        case .perpetuals:
            navigationState.wallet.append(Scenes.Perpetuals())

        case let .rewards(code):
            navigationState.settings.append(Scenes.Referral(code: code))

        case let .receive(assetId):
            try await presentReceive(assetId: AssetId(id: assetId))

        case let .buy(assetId, amount):
            try await presentFiat(type: .buy, assetId: AssetId(id: assetId), amount: amount.map(\.asInt))

        case let .sell(assetId, amount):
            try await presentFiat(type: .sell, assetId: AssetId(id: assetId), amount: amount.map(\.asInt))

        case let .swap(assetId):
            try await presentSwap(from: AssetId(id: assetId), to: .none)
        }

        selectTab(for: deeplink.selectTab)
    }
}

// MARK: - Payment

@MainActor
extension NavigationHandler {
    private func handlePayment(_ payment: Payment) async throws {
        guard let wallet = await walletSessionService.currentWallet else { return }
        switch payment {
        case let .request(request):
            let assets = try assetStore.getAssetsData(walletId: wallet.id, filters: [])
            presenter.isPresentingPayment.wrappedValue = try PaymentDestinationBuilder.build(payment: request, assets: assets, paymentService: paymentService)
        case let .link(link):
            toastPresenter.toastMessage = ToastMessage(title: Localized.Common.loading, image: SystemImage.network)
            let addresses = wallet.accounts.map { ChainAddress(chain: $0.chain, address: $0.address) }
            let transaction = try await paymentService.load(link: link, addresses: addresses)
            let asset = try await assetsService.ensureTokenAsset(for: Primitives.AssetId(core: paymentService.transactionAssetId(transaction: transaction)))
            toastPresenter.toastMessage = nil
            presenter.isPresentingPayment.wrappedValue = PaymentDestinationBuilder.build(transaction: transaction, asset: asset, paymentService: paymentService)
        }
    }
}

// MARK: - WalletConnect

@MainActor
extension NavigationHandler {
    private func handleWalletConnect(_ link: WalletConnectLink) async {
        walletConnectorPresenter.isPresentingConnectionBar = true

        do {
            switch link {
            case let .connect(uri):
                try await walletConnector.pair(uri: uri)
            case .request:
                break
            case .session:
                walletConnector.updateSessions()
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
    private func handle(_ notification: GemPushNotification) async throws {
        switch notification {
        case let .asset(assetId), let .priceAlert(assetId):
            try await navigateToAsset(Primitives.AssetId(id: assetId))
        case let .fiatTransaction(walletId, assetId), let .stake(walletId, assetId):
            try await navigateToAsset(walletId: Primitives.WalletId.from(id: walletId), assetId: Primitives.AssetId(id: assetId))
        case let .transaction(walletId, assetId, transaction):
            try await navigateToTransaction(
                walletId: Primitives.WalletId.from(id: walletId),
                assetId: Primitives.AssetId(id: assetId),
                transaction: Primitives.Transaction(transaction),
            )
        case let .buyAsset(assetId):
            try await presentFiat(type: .buy, assetId: Primitives.AssetId(id: assetId), amount: .none)
        case let .swapAsset(fromAssetId, toAssetId):
            try await presentSwap(from: Primitives.AssetId(id: fromAssetId), to: Primitives.AssetId(id: toAssetId))
        case .support:
            presenter.isPresentingSupport.wrappedValue = true
        case .rewards:
            navigationState.settings.append(Scenes.Referral(code: .none))
        case .test: break
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
        guard let asset = try await assetsService.openAsset(for: assetId) else {
            return
        }
        navigationState.openAsset(asset)
    }

    private func navigateToAsset(walletId: WalletId, assetId: AssetId) async throws {
        guard let wallet = try? await walletSessionService.getWallet(walletId: walletId),
              let asset = try await assetsService.openWalletAsset(wallet: wallet, assetId: assetId)
        else {
            return
        }

        await selectWalletIfNeeded(walletId)
        navigationState.openAsset(asset)
    }

    private func trackNotificationTransaction(walletId: WalletId, transaction: Primitives.Transaction) {
        Task {
            do {
                try await transactionStateService.track(walletId: walletId.id, transactions: [transaction.json()])
            } catch {
                debugLog("navigation: transaction tracking failed \(error)")
            }
        }
    }

    private func navigateToTransaction(walletId: WalletId, assetId: AssetId, transaction: Primitives.Transaction) async throws {
        guard let wallet = try? await walletSessionService.getWallet(walletId: walletId),
              let asset = try await transactionStateService.addNotificationTransaction(
                  wallet: wallet.json(),
                  assetId: assetId.identifier,
                  transaction: transaction.json(),
              ).map({ $0.map() })
        else {
            return
        }
        trackNotificationTransaction(walletId: walletId, transaction: transaction)
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
        guard let wallet = await walletSessionService.currentWallet else { return }
        try await presenter.presentSwap(from: fromId, to: toId, wallet: wallet)
    }

    private func presentFiat(type: FiatQuoteType, assetId: AssetId, amount: Int?) async throws {
        let asset = try await assetsService.ensureAsset(for: assetId)
        let selectedType: SelectedAssetType = switch type {
        case .buy: .buy(asset, amount: amount)
        case .sell: .sell(asset, amount: amount)
        }
        try await presentAssetInput(type: selectedType, for: asset)
    }

    private func presentReceive(assetId: AssetId) async throws {
        let asset = try await assetsService.ensureAsset(for: assetId)
        try await presentAssetInput(type: .receive(.asset), for: asset)
    }

    private func presentAssetInput(type: SelectedAssetType, for asset: Asset) async throws {
        guard let wallet = await walletSessionService.currentWallet else { return }
        try presenter.presentAssetInput(type: type, for: asset, wallet: wallet)
    }

    func resetNavigation() {
        navigationState.clearAll()
        navigationState.selectedTab = .wallet
    }
}

// MARK: - TabItem Selection

private extension Deeplink {
    var selectTab: TabItem? {
        switch self {
        case .asset, .perpetuals: .wallet
        case .rewards: .settings
        case .receive, .buy, .sell, .swap: nil
        }
    }
}

private extension GemPushNotification {
    var selectTab: TabItem? {
        switch self {
        case .transaction, .asset, .fiatTransaction, .priceAlert, .stake: .wallet
        case .buyAsset, .swapAsset: nil
        case .support, .rewards: .settings
        case .test: nil
        }
    }
}
