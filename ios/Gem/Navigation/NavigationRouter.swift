// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.Deeplink
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemDeeplinkServiceProtocol
import protocol Gemstone.GemNavigationServiceProtocol
import enum Gemstone.GemNavigationTab
import enum Gemstone.GemNavigationTarget
import protocol Gemstone.GemPaymentServiceProtocol
import enum Gemstone.GemPaymentTarget
import enum Gemstone.GemPushNotification
import protocol Gemstone.GemPushNotificationServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import protocol Gemstone.GemWalletSessionServiceProtocol
import enum Gemstone.Payment
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
import Transfer
import WalletConnector
import WalletConnectorService

@Observable
final class NavigationRouter: Sendable {
    private let navigationState: NavigationStateManager
    private let presenter: NavigationPresenter

    private let assetsService: any GemAssetsServiceProtocol
    private let assetStore: AssetStore
    private let walletConnector: any WalletConnectorServiceable
    private let toastPresenter: ToastPresenter
    private let pushNotificationService: any GemPushNotificationServiceProtocol
    private let transactionStore: TransactionStore
    private let deeplinkService: any GemDeeplinkServiceProtocol
    private let navigationService: any GemNavigationServiceProtocol
    private let paymentService: any GemPaymentServiceProtocol
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
        deeplinkService: any GemDeeplinkServiceProtocol,
        navigationService: any GemNavigationServiceProtocol,
        paymentService: any GemPaymentServiceProtocol,
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
        self.navigationService = navigationService
        self.paymentService = paymentService
        self.transactionStateService = transactionStateService
        self.walletConnectorPresenter = walletConnectorPresenter
        self.walletSessionService = walletSessionService
    }

    @MainActor
    func openNotification(userInfo: [AnyHashable: Any]) async {
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
            try await open(notification: notification)
        } catch {
            debugLog("NavigationRouter push error: \(error)")
        }
    }

    private static func payload(_ data: Any?) -> String? {
        guard let data, JSONSerialization.isValidJSONObject(data) else { return .none }
        return (try? JSONSerialization.data(withJSONObject: data)).map { String(decoding: $0, as: UTF8.self) }
    }

    @MainActor
    func open(url: URL) async {
        await open(code: url.absoluteString)
    }

    @MainActor
    func open(code: String) async {
        guard let action = deeplinkService.urlAction(url: code) else {
            return showError(AnyError(Localized.Errors.notSupported))
        }
        await open(action: action)
    }

    @MainActor
    func open(action: UrlAction) async {
        do {
            try await openURLAction(action)
        } catch {
            toastPresenter.toastMessage = nil
            showError(error)
        }
    }

    @MainActor
    func openInApp(url: URL) -> Bool {
        guard let action = deeplinkService.urlAction(url: url.absoluteString) else { return false }
        Task { await open(action: action) }
        return true
    }
}

// MARK: - UrlAction

@MainActor
extension NavigationRouter {
    private func openURLAction(_ action: UrlAction) async throws {
        switch action {
        case let .deeplink(deeplink): try await openDeeplink(deeplink)
        case let .payment(payment): try await openPayment(payment)
        case let .walletConnect(link): await openWalletConnect(link)
        }
    }

    private func openDeeplink(_ deeplink: Deeplink) async throws {
        try await open(target: navigationService.openDeeplink(deeplink: deeplink))
    }

    private func open(target: GemNavigationTarget) async throws {
        switch target {
        case let .asset(asset, walletId, _):
            try openTarget(path: getPath(for: asset.toPrimitives()), walletId: walletId)
        case let .receive(asset):
            try await presentAssetInput(type: .receive(.asset), for: asset.toPrimitives())
        case let .fiat(asset, amount, quoteType):
            let asset = asset.toPrimitives()
            let type: SelectedAssetType = switch quoteType.toPrimitives() {
            case .buy: .buy(asset, amount: amount.map(Int.init))
            case .sell: .sell(asset, amount: amount.map(Int.init))
            }
            try await presentAssetInput(type: type, for: asset)
        case let .swap(from, to):
            try await presentSwap(from: from.toPrimitives().id, to: to?.toPrimitives().id)
        case .perpetuals:
            navigationState.wallet.append(Scenes.Perpetuals())
        case let .rewards(code):
            navigationState.settings.append(Scenes.Referral(code: code))
        case .support:
            presenter.isPresentingSupport.wrappedValue = true
        case let .transaction(asset, walletId, transaction, _):
            let stored = try transactionStore.getTransaction(walletId: Primitives.WalletId.from(id: walletId), transactionId: transaction.toPrimitives().id)
            try openTarget(path: getPath(for: asset.toPrimitives(), transaction: stored), walletId: walletId)
        case .none:
            break
        }
        selectTab(target.tab())
    }

    private func openTarget(path: [any Hashable & Codable], walletId: String?) throws {
        guard let walletId else {
            return navigationState.openWallet(path: path)
        }
        try openWallet(Primitives.WalletId.from(id: walletId), path: path)
    }
}

// MARK: - Payment

@MainActor
extension NavigationRouter {
    private func openPayment(_ payment: Gemstone.Payment) async throws {
        let wallet = try await walletSessionService.requireCurrentWallet()
        if case .link = payment {
            toastPresenter.toastMessage = ToastMessage(title: Localized.Common.loading, image: SystemImage.network)
        }
        let target = try await paymentService.prepare(payment: payment, wallet: wallet)
        toastPresenter.toastMessage = nil
        presenter.isPresentingPayment.wrappedValue = try paymentDestination(target, wallet: wallet.toPrimitives())
    }

    private func paymentDestination(_ target: GemPaymentTarget, wallet: Primitives.Wallet) throws -> PaymentDestination {
        switch target {
        case let .confirm(transfer):
            return .confirm(transfer)
        case let .verify(url, link):
            guard let url = URL(string: url) else {
                throw AnyError(Localized.Errors.notSupported)
            }
            return .verify(url, link: link)
        case let .recipient(asset, payment):
            let asset = asset.toPrimitives()
            guard let assetData = try assetStore.getAssetsData(walletId: wallet.id, filters: [.chainsOrAssets([], [asset.id.identifier])]).first else {
                throw AnyError(Localized.Errors.notSupported)
            }
            return .recipient(
                SelectedAssetInput(
                    type: .send(.asset(asset: asset.toGem())),
                    assetData: assetData,
                    recipient: payment,
                ),
            )
        case let .selectAsset(payment, chains):
            return .selectAsset(.send(payment), chains: chains.compactMap { Primitives.Chain(rawValue: $0) })
        case .unsupported:
            throw AnyError(Localized.Errors.notSupported)
        }
    }
}

// MARK: - WalletConnect

@MainActor
extension NavigationRouter {
    private func openWalletConnect(_ link: WalletConnectLink) async {
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
            debugLog("NavigationRouter walletConnect error: \(error)")
            walletConnectorPresenter.isPresentingError = error.localizedDescription
        }
    }
}

// MARK: - PushNotification

@MainActor
extension NavigationRouter {
    private func open(notification: GemPushNotification) async throws {
        try await open(target: navigationService.openNotification(notification: notification))
    }
}

// MARK: - Private

@MainActor
extension NavigationRouter {
    private func showError(_ error: any Error) {
        debugLog("NavigationRouter error: \(error)")
        toastPresenter.toastMessage = .error(error.localizedDescription)
    }

    private func selectTab(_ tab: GemNavigationTab?) {
        switch tab {
        case .wallet: navigationState.selectedTab = .wallet
        case .settings: navigationState.selectedTab = .settings
        case .none: break
        }
    }

    private func openWallet(_ walletId: WalletId, path: [any Hashable & Codable]) throws {
        guard walletSessionService.currentWalletId != walletId else {
            return navigationState.openWallet(path: path)
        }
        try walletSessionService.setCurrent(walletId: walletId)
        navigationState.pendingWalletPath = path
    }

    private func getPath(for asset: Asset) -> [any Hashable & Codable] {
        switch asset.type {
        case .perpetual: [Scenes.Perpetual(asset)]
        default: [Scenes.Asset(asset: asset)]
        }
    }

    private func getPath(for asset: Asset, transaction: TransactionExtended) -> [any Hashable & Codable] {
        switch asset.type {
        case .perpetual: [Scenes.Perpetuals(), Scenes.Perpetual(asset), Scenes.Transaction(transaction: transaction)]
        default: [Scenes.Asset(asset: asset), Scenes.Transaction(transaction: transaction)]
        }
    }

    private func presentSwap(from fromId: AssetId, to toId: AssetId?) async throws {
        let wallet = try await walletSessionService.requireCurrentWallet().toPrimitives()
        try await presenter.presentSwap(from: fromId, to: toId, wallet: wallet)
    }

    private func presentAssetInput(type: SelectedAssetType, for asset: Asset) async throws {
        let wallet = try await walletSessionService.requireCurrentWallet().toPrimitives()
        try presenter.presentAssetInput(type: type, for: asset, wallet: wallet)
    }

    func resetNavigation() {
        navigationState.reset()
    }
}
