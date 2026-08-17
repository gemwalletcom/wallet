// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import Localization
import PriceAlerts
import Primitives
import Style
import SwiftUI
import Transactions
import TransactionsService
import WalletTab

struct MainTabView: View {
    @Environment(\.assetDiscoveryService) private var assetDiscoveryService
    @Environment(\.balanceService) private var balanceService
    @Environment(\.bannerService) private var bannerService
    @Environment(\.deviceService) private var deviceService
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.nftService) private var nftService
    @Environment(\.priceService) private var priceService
    @Environment(\.observablePreferences) private var observablePreferences
    @Environment(\.assetsService) private var assetsService
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.transactionsService) private var transactionsService
    @Environment(\.viewModelFactory) private var viewModelFactory

    let wallet: Wallet

    @State private var model: MainTabViewModel

    private var tabViewSelection: Binding<TabItem> {
        Binding(
            get: { navigationState.selectedTab },
            set: { onSelect(tab: $0) },
        )
    }

    init(wallet: Wallet) {
        self.wallet = wallet
        _model = State(initialValue: MainTabViewModel(wallet: wallet))
    }

    var body: some View {
        TabView(selection: tabViewSelection) {
            NavigationStack(path: navigationState.wallet.binding) {
                WalletNavigationView(
                    model: WalletSceneViewModel(
                        assetDiscoveryService: assetDiscoveryService,
                        balanceService: balanceService,
                        bannerService: bannerService,
                        nftService: nftService,
                        observablePreferences: observablePreferences,
                        wallet: wallet,
                        isPresentingSelectedAssetInput: presenter.isPresentingAssetInput,
                        isPresentingWallets: presenter.isPresentingWallets,
                    ),
                )
                .id(wallet.id)
            }
            .tabItem {
                tabItem(Localized.Wallet.title, Images.Tabs.wallet)
            }
            .tag(TabItem.wallet)

            if model.isMarketEnabled {
                MarketsNavigationStack()
                    .tabItem {
                        tabItem("Markets", Images.Tabs.markets)
                    }
                    .tag(TabItem.markets)
            }

            NavigationStack(path: navigationState.activity.binding) {
                TransactionsNavigationView(
                    model: TransactionsViewModel(
                        transactionsService: transactionsService,
                        wallet: wallet,
                        type: .all,
                    ),
                )
                .id(wallet.id)
            }
            .tabItem {
                tabItem(Localized.Activity.title, Images.Tabs.activity)
            }
            .badge(model.transactions)
            .tag(TabItem.activity)

            NavigationStack(path: navigationState.settings.binding) {
                SettingsNavigationView(
                    walletId: wallet.id,
                    priceService: priceService,
                    deviceService: deviceService,
                    isPresentingSupport: presenter.isPresentingSupport,
                )
                .id(wallet.id)
            }
            .tabItem {
                tabItem(Localized.Settings.title, Images.Tabs.settings)
            }
            .tag(TabItem.settings)
        }
        .sheet(item: presenter.isPresentingAssetInput) { input in
            SelectedAssetNavigationStack(
                input: input,
                wallet: wallet,
                onComplete: { onComplete(type: input.type) },
            )
        }
        .sheet(item: presenter.isPresentingPayment) { input in
            switch input {
            case let .confirm(data):
                ConfirmTransferNavigationStack(wallet: wallet, transferData: data, onComplete: onPaymentComplete)
            case let .recipient(assetInput):
                SelectedAssetNavigationStack(input: assetInput, wallet: wallet, onComplete: onPaymentComplete)
            case let .selectAsset(type, chains):
                SelectAssetSceneNavigationStack(
                    model: viewModelFactory.selectAssetScene(wallet: wallet, selectType: type, chains: chains),
                )
            }
        }
        .sheet(item: presenter.isPresentingPriceAlert) { input in
            SetPriceAlertNavigationStack(
                model: SetPriceAlertViewModel(
                    walletId: wallet.id,
                    asset: input.asset,
                    priceAlertService: priceAlertService,
                    price: input.price,
                    onComplete: onSetPriceAlertComplete,
                ),
            )
        }
        .toast(message: $model.isPresentingToastMessage)
        .bindQuery(model.transactionsQuery)
        .task(id: wallet.id) { model.transactionsQuery.request.walletId = wallet.id }
        .connectionStatusBanner()
    }
}

// MARK: - UI Components

extension MainTabView {
    private func tabItem(_ title: String, _ image: Image) -> Label<Text, Image> {
        Label(
            title: { Text(title) },
            icon: { image },
        )
    }
}

// MARK: - Actions

extension MainTabView {
    private func onSelect(tab: TabItem) {
        navigationState.select(tab: tab)
    }

    private func onPaymentComplete() {
        presenter.isPresentingPayment.wrappedValue = nil
    }

    private func onSetPriceAlertComplete(message: String) {
        presenter.isPresentingPriceAlert.wrappedValue = nil
        model.isPresentingToastMessage = .priceAlert(message: message)
    }

    private func onComplete(type: SelectedAssetType) {
        switch type {
        case .receive, .stake, .earn, .buy, .sell:
            presenter.isPresentingAssetInput.wrappedValue = nil
        case let .send(type):
            switch type {
            case .nft:
                navigationState.activity.reset()
                navigationState.selectedTab = .activity
            case .asset:
                break
            }
            presenter.isPresentingAssetInput.wrappedValue = nil
        case let .swap(fromAsset, _):
            Task {
                let asset = try await assetsService.getOrFetchAsset(for: fromAsset.id)

                switch navigationState.selectedTab {
                case .wallet:
                    navigationState.wallet.setPath([Scenes.Asset(asset: asset)])
                case .activity:
                    navigationState.wallet.setPath([Scenes.Asset(asset: asset)])
                    navigationState.selectedTab = .wallet
                case .markets, .settings:
                    break
                }
                presenter.isPresentingAssetInput.wrappedValue = nil
            }
        }
    }
}
