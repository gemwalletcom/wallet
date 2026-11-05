// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
import NFT
import PriceAlerts
import Primitives
import PrimitivesComponents
import Style
import Support
import SwiftUI
import Transactions
import Transfer
import Wallet

struct MainTabView: View {
    @Environment(\.navigationRouter) private var navigationRouter
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
            WalletNavigationStack(wallet: wallet)
                .tabItem {
                    tabItem(Localized.Wallet.title, Images.Tabs.wallet, id: "mainTab")
                }
                .tag(TabItem.wallet)

            ActivityNavigationStack(wallet: wallet)
                .tabItem {
                    tabItem(Localized.Activity.title, Images.Tabs.activity, id: "activitiesTab")
                }
                .badge(model.transactions)
                .tag(TabItem.activity)

            SettingsNavigationStack(wallet: wallet)
                .tabItem {
                    tabItem(Localized.Settings.title, Images.Tabs.settings, id: "settingsTab")
                }
                .tag(TabItem.settings)
        }
        .sheet(item: presenter.sheet) { sheet in
            sheetView(sheet)
                .id(sheet.id)
        }
        .bindQuery(model.transactionsQuery)
        .task(id: wallet.id) { model.transactionsQuery.request.walletId = wallet.id }
        .connectionStatusBanner()
    }
}

// MARK: - UI Components

extension MainTabView {
    private func tabItem(_ title: String, _ image: Image, id: String) -> some View {
        Label(
            title: { Text(title) },
            icon: { image },
        )
        .accessibilityIdentifier(id)
    }

    @ViewBuilder
    private func sheetView(_ sheet: NavigationSheetType) -> some View {
        switch sheet {
        case let .assetInput(input):
            SelectedAssetNavigationStack(
                input: input,
                wallet: wallet,
                onComplete: { navigationRouter.completeAssetInput(input.type) },
            )
        case let .payment(destination):
            PaymentNavigationStack(type: destination, wallet: wallet)
        case let .priceAlert(asset):
            SetPriceAlertNavigationStack(
                model: viewModelFactory.setPriceAlertScene(walletId: wallet.id, asset: asset, onComplete: navigationRouter.completePriceAlert),
            )
        case let .addressDetails(chainAddress):
            AddressDetailsDestination(chainAddress: chainAddress)
        case let .addContact(action):
            AddContactNavigationStack(action: action)
        case .support:
            NavigationStack {
                SupportChatScene(model: viewModelFactory.supportChatScene())
                    .toolbarDismissItem(type: .close, placement: .topBarLeading)
            }
            .environment(\.openURL, OpenURLAction { url in
                guard navigationRouter.openInApp(url: url) else { return .systemAction }
                presenter.dismiss()
                return .handled
            })
        case .wallets:
            WalletsNavigationStack()
        case let .wallet(sheet):
            walletSheetView(sheet)
        }
    }

    @ViewBuilder
    private func walletSheetView(_ sheet: WalletSheetType) -> some View {
        switch sheet {
        case let .selectAsset(type, chains):
            SelectAssetNavigationStack(
                model: viewModelFactory.selectAssetScene(wallet: wallet, selectType: type, chains: chains),
            )
        case let .amount(input):
            AmountNavigationStack(input: input, wallet: wallet, onComplete: presenter.dismiss)
        case let .infoSheet(type):
            InfoSheetScene(sheet: type)
        case let .transferData(data):
            ConfirmTransferNavigationStack(wallet: wallet, transferData: data, onComplete: presenter.dismiss)
        case .addAsset:
            AddAssetNavigationStack(wallet: wallet)
        case let .portfolio(defaultType):
            PortfolioScene(model: viewModelFactory.portfolioScene(wallet: wallet, defaultType: defaultType))
        case .swap:
            SwapNavigationStack(wallet: wallet, onComplete: presenter.dismiss)
        }
    }
}

// MARK: - Actions

extension MainTabView {
    private func onSelect(tab: TabItem) {
        navigationState.select(tab: tab)
    }
}
