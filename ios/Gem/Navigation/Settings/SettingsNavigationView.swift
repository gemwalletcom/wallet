// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import GemstoneServices
import InAppNotifications
import Market
import PriceAlerts
import Primitives
import PrimitivesComponents
import Rewards
import Settings
import Store
import Support
import SwiftUI
import WalletConnector
import WalletConnectorService

struct SettingsNavigationView: View {
    @Environment(\.navigationRouter) private var navigationRouter
    @Environment(\.walletConnector) private var walletConnector
    @Environment(\.walletConnectorPresenter) private var walletConnectorPresenter
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.navigationPresenter) private var presenter

    @State private var currencyModel: CurrencySceneViewModel
    @State private var isPresentingAddressDetails: ChainAddress?

    let walletId: WalletId
    @Binding var isPresentingSupport: Bool

    init(
        walletId: WalletId,
        viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory,
        isPresentingSupport: Binding<Bool>,
    ) {
        self.walletId = walletId
        _isPresentingSupport = isPresentingSupport
        _currencyModel = State(initialValue: viewModelFactory.currencyScene())
    }

    var body: some View {
        SettingsScene(
            model: viewModelFactory.settingsScene(),
            isPresentingWallets: presenter.isPresentingWallets,
            isPresentingSupport: $isPresentingSupport,
        )
        .navigationBarTitleDisplayMode(.inline)
        .navigationDestination(for: Scenes.Security.self) { _ in
            SecurityScene(model: viewModelFactory.securityScene())
        }
        .navigationDestination(for: Scenes.Notifications.self) { _ in
            NotificationsScene(
                model: viewModelFactory.notificationsScene(),
            )
        }
        .navigationDestination(for: Scenes.PriceAlerts.self) { _ in
            PriceAlertsNavigationView(
                model: viewModelFactory.priceAlertsScene(),
            )
        }
        .navigationDestination(for: Scenes.AssetPriceAlert.self) {
            AssetPriceAlertsScene(
                model: viewModelFactory.assetPriceAlertsScene(walletId: walletId, asset: $0.asset),
            )
        }
        .navigationDestination(for: Scenes.Chart.self) { scene in
            ChartScene(
                model: viewModelFactory.chartScene(
                    asset: scene.asset,
                    onSetPriceAlert: { presenter.isPresentingPriceAlert.wrappedValue = $0 },
                    onSelectAddress: { isPresentingAddressDetails = $0 },
                ),
            )
        }
        .navigationDestination(for: Scenes.Chains.self) { _ in
            ChainListSettingsScene(model: viewModelFactory.chainListSettingsScene())
        }
        .navigationDestination(for: Scenes.ServiceStatus.self) { _ in
            ServiceStatusScene(model: viewModelFactory.serviceStatusScene())
        }
        .navigationDestination(for: Scenes.AboutUs.self) { _ in
            AboutUsScene(
                model: viewModelFactory.aboutUsScene(),
            )
        }
        .navigationDestination(for: Scenes.WalletConnect.self) { _ in
            ConnectionsScene(
                model: viewModelFactory.connectionsScene(
                    connector: walletConnector,
                    walletConnectorPresenter: walletConnectorPresenter,
                ),
            )
        }
        .navigationDestination(for: Scenes.Developer.self) { _ in
            DeveloperScene(model: viewModelFactory.developerScene(walletId: walletId))
        }
        .navigationDestination(for: Scenes.DeveloperPayments.self) { _ in
            DeveloperPaymentsScene { payload in
                Task { await navigationRouter.open(code: payload) }
            }
        }
        .navigationDestination(for: Scenes.InAppNotifications.self) { _ in
            if let model = viewModelFactory.inAppNotificationsScene() {
                InAppNotificationsScene(model: model)
            }
        }
        .navigationDestination(for: Scenes.Currency.self) { _ in
            CurrencyScene(model: currencyModel)
        }
        .navigationDestination(for: Scenes.Preferences.self) { _ in
            PreferencesScene(model: viewModelFactory.preferencesScene())
        }
        .navigationDestination(for: Scenes.Appearance.self) { _ in
            AppearanceScene(model: viewModelFactory.appearanceScene())
        }
        .navigationDestination(for: Scenes.Rewards.self) { scene in
            if let model = viewModelFactory.rewardsScene(activateCode: scene.code) {
                RewardsScene(model: model)
            }
        }
        .navigationDestination(for: Scenes.ChainSettings.self) {
            ChainSettingsScene(model: viewModelFactory.chainSettingsScene(chain: $0.chain))
        }
        .navigationDestination(for: Scenes.Contacts.self) { _ in
            ContactsNavigationView(model: viewModelFactory.contactsScene())
        }
        .sheet(isPresented: Binding(get: { isPresentingAddressDetails != nil }, set: {
            if !$0 {
                isPresentingAddressDetails = nil
            }
        })) {
            if let isPresentingAddressDetails {
                AddressDetailsDestination(chainAddress: isPresentingAddressDetails)
            }
        }
        .sheet(isPresented: $isPresentingSupport) {
            NavigationStack {
                SupportChatScene(model: viewModelFactory.supportChatScene())
                    .toolbarDismissItem(type: .close, placement: .topBarLeading)
            }
            .environment(\.openURL, OpenURLAction { url in
                guard navigationRouter.openInApp(url: url) else { return .systemAction }
                isPresentingSupport = false
                return .handled
            })
        }
    }
}
