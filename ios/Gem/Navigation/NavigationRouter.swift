// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAssetDestination
import enum Gemstone.GemCodeOutcome
import protocol Gemstone.GemDeeplinkServiceProtocol
import protocol Gemstone.GemNavigationServiceProtocol
import enum Gemstone.GemNavigationTab
import enum Gemstone.GemNavigationTarget
import protocol Gemstone.GemPaymentServiceProtocol
import enum Gemstone.GemPaymentTarget
import enum Gemstone.GemTransactionHeaderAction
import protocol Gemstone.GemWalletSessionServiceProtocol
import enum Gemstone.Payment
import enum Gemstone.UrlAction
import enum Gemstone.WalletConnectLink
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Settings
import Style
import SwiftUI
import Transfer
import WalletConnector
import WalletConnectorService

@Observable
final class NavigationRouter: Sendable {
    private let navigationState: NavigationStateManager
    private let presenter: NavigationPresenter

    private let walletConnector: any WalletConnectorServiceable
    private let toastPresenter: ToastPresenter
    private let deeplinkService: any GemDeeplinkServiceProtocol
    private let navigationService: any GemNavigationServiceProtocol
    private let paymentService: any GemPaymentServiceProtocol
    private let walletConnectorPresenter: WalletConnectorPresenter
    private let walletSessionService: any GemWalletSessionServiceProtocol
    private let lockModel: LockSceneViewModel

    init(
        navigationState: NavigationStateManager,
        presenter: NavigationPresenter,
        walletConnector: any WalletConnectorServiceable,
        toastPresenter: ToastPresenter,
        deeplinkService: any GemDeeplinkServiceProtocol,
        navigationService: any GemNavigationServiceProtocol,
        paymentService: any GemPaymentServiceProtocol,
        walletConnectorPresenter: WalletConnectorPresenter,
        walletSessionService: any GemWalletSessionServiceProtocol,
        lockModel: LockSceneViewModel,
    ) {
        self.navigationState = navigationState
        self.presenter = presenter
        self.walletConnector = walletConnector
        self.toastPresenter = toastPresenter
        self.deeplinkService = deeplinkService
        self.navigationService = navigationService
        self.paymentService = paymentService
        self.walletConnectorPresenter = walletConnectorPresenter
        self.walletSessionService = walletSessionService
        self.lockModel = lockModel
    }

    @MainActor
    func openNotification(userInfo: [AnyHashable: Any]) async {
        await lockModel.waitUntilUnlocked()
        guard let notificationType = userInfo["type"] as? String else { return }
        do {
            try await open(target: navigationService.openNotification(
                notificationType: notificationType,
                data: Self.payload(userInfo["data"]),
            ))
        } catch {
            showError(error)
        }
    }

    private static func payload(_ data: Any?) -> String? {
        guard let data, JSONSerialization.isValidJSONObject(data) else { return .none }
        return (try? JSONSerialization.data(withJSONObject: data)).map { String(decoding: $0, as: UTF8.self) }
    }

    @MainActor
    func open(url: URL) async {
        await lockModel.waitUntilUnlocked()
        let outcome = await navigationService.openCode(code: url.absoluteString)
        if case let .walletConnect(link) = outcome {
            await walletConnector.linkOpened(link)
        }
        await open(outcome: outcome)
    }

    @MainActor
    func open(code: String) async {
        await open(outcome: navigationService.openCode(code: code))
    }

    @MainActor
    func open(action: UrlAction) async {
        await open(outcome: navigationService.openAction(action: action))
    }

    @MainActor
    func openAsset(_ asset: Asset) {
        Task { await open(assetId: asset.id) }
    }

    @MainActor
    private func open(assetId: AssetId) async {
        do {
            try await appendAsset(target: navigationService.openAsset(assetId: assetId))
        } catch {
            showError(error)
        }
    }

    @MainActor
    func openInApp(url: URL) -> Bool {
        guard let action = deeplinkService.urlAction(url: url.absoluteString) else { return false }
        Task { await open(action: action) }
        return true
    }

    @MainActor
    private func open(outcome: GemCodeOutcome) async {
        do {
            switch outcome {
            case let .open(target): try await open(target: target)
            case let .walletConnect(link): await openWalletConnect(link)
            case let .payment(payment, showsLoading): try await openPayment(payment, showsLoading: showsLoading)
            case let .failure(text): showError(message: text.text)
            }
        } catch {
            toastPresenter.toastMessage = nil
            showError(error)
        }
    }
}

// MARK: - Target

@MainActor
extension NavigationRouter {
    private func open(target: GemNavigationTarget) async throws {
        if case .none = target {
            return
        }
        presenter.dismiss()
        switch target {
        case let .asset(destination, walletId):
            try openTarget(path: Self.assetPath(destination), walletId: walletId)
        case let .chart(asset):
            navigationState.openWallet(path: [Scenes.Chart(asset: asset.toPrimitives())])
        case let .receive(asset, account):
            presentAssetInput(type: .receive(.asset), asset: asset.toPrimitives(), account: account.toPrimitives())
        case let .fiat(asset, account, amount, quoteType):
            let asset = asset.toPrimitives()
            let type: SelectedAssetType = switch quoteType.toPrimitives() {
            case .buy: .buy(asset, amount: amount.map(Int.init))
            case .sell: .sell(asset, amount: amount.map(Int.init))
            }
            presentAssetInput(type: type, asset: asset, account: account.toPrimitives())
        case let .swap(from, account, to):
            let from = from.toPrimitives()
            presentAssetInput(type: .swap(from.id, to?.toPrimitives().id), asset: from, account: account.toPrimitives())
        case .perpetuals:
            navigationState.openWallet(path: [Scenes.Perpetuals()])
        case let .rewards(code):
            navigationState.settings.setPath([Scenes.Rewards(code: code)])
        case .support:
            presenter.sheet.wrappedValue = .support
        case let .address(chain, address):
            presenter.sheet.wrappedValue = .addressDetails(ChainAddress(chain: Chain(core: chain), address: address))
        case let .transaction(destination, walletId, transactionId):
            try openTarget(path: Self.assetPath(destination) + [Scenes.Transaction(id: transactionId)], walletId: walletId)
        case .none:
            break
        }
        selectTab(target.tab())
    }

    func openTransactionAction(_ action: GemTransactionHeaderAction, collectiblePath: NavigationPathState) async throws {
        switch action {
        case let .asset(assetId), let .perpetual(assetId):
            try await appendAsset(target: navigationService.openAsset(assetId: assetId))
        case let .swap(fromAssetId, toAssetId):
            try await open(target: navigationService.openSwap(fromAssetId: fromAssetId, toAssetId: toAssetId))
        case let .nft(assetId):
            try collectiblePath.append(Scenes.Collectible.assetId(NFTAssetId.from(id: assetId)))
        }
    }

    private func appendAsset(target: GemNavigationTarget) {
        guard case let .asset(destination, _) = target else { return }
        navigationState.appendWallet(Self.assetScene(destination))
    }

    private static func assetScene(_ destination: GemAssetDestination) -> any Hashable & Codable {
        switch destination {
        case let .asset(asset): Scenes.Asset(asset: asset.toPrimitives())
        case let .perpetual(asset): Scenes.Perpetual(asset.toPrimitives())
        }
    }

    private static func assetPath(_ destination: GemAssetDestination) -> [any Hashable & Codable] {
        switch destination {
        case .asset: [assetScene(destination)]
        case .perpetual: [Scenes.Perpetuals(), assetScene(destination)]
        }
    }

    private func openTarget(path: [any Hashable & Codable], walletId: WalletId?) throws {
        guard let walletId else {
            return navigationState.openWallet(path: path)
        }
        try openWallet(walletId, path: path)
    }
}

// MARK: - Payment

@MainActor
extension NavigationRouter {
    private func openPayment(_ payment: Gemstone.Payment, showsLoading: Bool) async throws {
        let wallet = try await walletSessionService.requireCurrentWallet()
        if showsLoading {
            toastPresenter.toastMessage = ToastMessage(title: Localized.Common.loading, image: SystemImage.network)
        }
        let target = try await paymentService.prepare(payment: payment, wallet: wallet)
        toastPresenter.toastMessage = nil
        let destination = try paymentDestination(target)
        presenter.sheet.wrappedValue = .payment(destination)
    }

    private func paymentDestination(_ target: GemPaymentTarget) throws -> PaymentDestination {
        switch target {
        case let .confirm(transfer):
            return .confirm(transfer)
        case let .verify(url, link):
            guard let url = URL(string: url) else {
                throw AnyError(Localized.Errors.notSupported)
            }
            return .verify(url, link: link)
        case let .amount(asset, payment):
            return .amount(AmountInput(type: .transfer(recipient: payment), asset: asset.toPrimitives()))
        case let .recipient(asset, account, payment):
            let asset = asset.toPrimitives()
            return .recipient(
                SelectedAssetInput(
                    type: .send(.asset(asset: asset.toGem())),
                    assetData: .with(asset: asset, account: account.toPrimitives()),
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
        do {
            switch link {
            case let .connect(uri):
                walletConnectorPresenter.isPresentingConnectionBar = true
                try await walletConnector.pair(uri: uri)
            case .request:
                break
            case .session:
                try await walletConnector.updateSessions()
            }
        } catch {
            debugLog("NavigationRouter walletConnect error: \(error)")
            walletConnectorPresenter.presentError(error.localizedDescription)
        }
    }
}

// MARK: - Private

@MainActor
extension NavigationRouter {
    private func showError(_ error: any Error) {
        debugLog("NavigationRouter error: \(error)")
        showError(message: error.localizedDescription)
    }

    func showError(message: String) {
        toastPresenter.toastMessage = .error(message)
    }

    private func selectTab(_ tab: GemNavigationTab?) {
        switch tab {
        case .wallet: navigationState.selectedTab = .wallet
        case .settings: navigationState.selectedTab = .settings
        case .none: break
        }
    }

    private func openWallet(_ walletId: WalletId, path: [any Hashable & Codable]) throws {
        guard try walletSessionService.getCurrentWalletId() != walletId else {
            return navigationState.openWallet(path: path)
        }
        try walletSessionService.setCurrentWalletId(walletId: walletId)
        navigationState.pendingWalletPath = path
    }

    private func presentAssetInput(type: SelectedAssetType, asset: Asset, account: Account) {
        presenter.sheet.wrappedValue = .assetInput(SelectedAssetInput(type: type, assetData: .with(asset: asset, account: account)))
    }

    func resetNavigation() {
        presenter.dismiss()
        navigationState.reset()
    }

    func backToRoot(tab: TabItem) {
        navigationState.backToRoot(tab: tab)
    }
}

// MARK: - Sheet completion

@MainActor
extension NavigationRouter {
    func completeAssetInput(_ type: SelectedAssetType) {
        switch type {
        case .receive, .stake, .earn, .buy, .sell, .send(.asset):
            presenter.dismiss()
        case .send(.nft):
            presenter.dismiss()
            navigationState.activity.reset()
            navigationState.selectedTab = .activity
        case let .swap(fromAssetId, _):
            Task { await completeSwap(fromAssetId: fromAssetId) }
        }
    }

    func completePriceAlert(message: String) {
        presenter.dismiss()
        toastPresenter.toastMessage = .priceAlert(message: message)
    }

    private func completeSwap(fromAssetId: AssetId) async {
        do {
            guard case let .asset(destination, _) = try await navigationService.openAsset(assetId: fromAssetId) else { return }
            navigationState.openWallet(path: Self.assetPath(destination))
            presenter.dismiss()
        } catch {
            showError(error)
        }
    }
}
