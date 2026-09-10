// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import Localization
import PriceAlerts
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import Transactions
import Transfer
import GemstoneServices
import WalletTab

struct MainTabView: View {
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationPresenter) private var presenter
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
                    model: viewModelFactory.walletScene(
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

            NavigationStack(path: navigationState.activity.binding) {
                TransactionsNavigationView(
                    model: viewModelFactory.transactionsScene(wallet: wallet, type: .all),
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
        .sheet(item: presenter.isPresentingPayment) { PaymentNavigationStack(type: $0, wallet: wallet) }
        .sheet(item: presenter.isPresentingPriceAlert) { asset in
            SetPriceAlertNavigationStack(
                model: viewModelFactory.setPriceAlertScene(walletId: wallet.id, asset: asset, onComplete: onSetPriceAlertComplete),
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
                try await presenter.completeSwap(fromAsset: fromAsset, navigationState: navigationState)
            }
        }
    }
}
