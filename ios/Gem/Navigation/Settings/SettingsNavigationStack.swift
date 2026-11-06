// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Components
import Contacts
import GemstoneServices
import InAppNotifications
import PriceAlerts
import Primitives
import PrimitivesComponents
import Rewards
import Settings
import Store
import SwiftUI
import WalletConnector
import WalletConnectorService

struct SettingsNavigationStack: View {
    @Environment(\.navigationRouter) private var navigationRouter
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.walletConnector) private var walletConnector
    @Environment(\.walletConnectorPresenter) private var walletConnectorPresenter
    @Environment(\.viewModelFactory) private var viewModelFactory

    let wallet: Wallet

    @State private var currencyModel: CurrencySceneViewModel

    init(wallet: Wallet, viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory) {
        self.wallet = wallet
        _currencyModel = State(initialValue: viewModelFactory.currencyScene())
    }

    var body: some View {
        NavigationStack(path: navigationState.settings.binding) {
            SettingsScene(
                model: viewModelFactory.settingsScene(),
                isPresentingWallets: presenter.isPresentingWallets,
                isPresentingSupport: presenter.isPresentingSupport,
            )
            .navigationBarTitleDisplayMode(.inline)
            .id(wallet.id)
            .navigationDestination(keyedBy: Scenes.Security.self, in: wallet.id) { _ in
                SecurityScene(model: viewModelFactory.securityScene())
            }
            .navigationDestination(keyedBy: Scenes.Notifications.self, in: wallet.id) { _ in
                NotificationsScene(
                    model: viewModelFactory.notificationsScene(),
                )
            }
            .navigationDestination(keyedBy: Scenes.PriceAlerts.self, in: wallet.id) { _ in
                PriceAlertsNavigationView(
                    model: viewModelFactory.priceAlertsScene(),
                )
            }
            .navigationDestination(keyedBy: Scenes.AssetPriceAlert.self, in: wallet.id) {
                AssetPriceAlertsScene(
                    model: viewModelFactory.assetPriceAlertsScene(walletId: wallet.id, asset: $0.asset),
                )
            }
            .navigationDestination(keyedBy: Scenes.Chart.self, in: wallet.id) { scene in
                ChartScene(
                    model: viewModelFactory.chartScene(
                        asset: scene.asset,
                        onSetPriceAlert: { presenter.sheet.wrappedValue = .priceAlert($0) },
                        onSelectAddress: { presenter.sheet.wrappedValue = .addressDetails($0) },
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.Chains.self, in: wallet.id) { _ in
                ChainListSettingsScene(model: viewModelFactory.chainListSettingsScene())
            }
            .navigationDestination(keyedBy: Scenes.ServiceStatus.self, in: wallet.id) { _ in
                ServiceStatusScene(model: viewModelFactory.serviceStatusScene())
            }
            .navigationDestination(keyedBy: Scenes.AboutUs.self, in: wallet.id) { _ in
                AboutUsScene(
                    model: viewModelFactory.aboutUsScene(),
                )
            }
            .navigationDestination(keyedBy: Scenes.WalletConnect.self, in: wallet.id) { _ in
                ConnectionsScene(
                    model: viewModelFactory.connectionsScene(
                        connector: walletConnector,
                        walletConnectorPresenter: walletConnectorPresenter,
                    ),
                )
            }
            .navigationDestination(keyedBy: Scenes.Developer.self, in: wallet.id) { _ in
                DeveloperScene(model: viewModelFactory.developerScene(walletId: wallet.id))
            }
            .navigationDestination(keyedBy: Scenes.DeveloperPayments.self, in: wallet.id) { _ in
                DeveloperPaymentsScene { payload in
                    Task { await navigationRouter.open(code: payload) }
                }
            }
            .navigationDestination(keyedBy: Scenes.InAppNotifications.self, in: wallet.id) { _ in
                if let model = viewModelFactory.inAppNotificationsScene() {
                    InAppNotificationsScene(model: model)
                }
            }
            .navigationDestination(keyedBy: Scenes.Currency.self, in: wallet.id) { _ in
                CurrencyScene(model: currencyModel)
            }
            .navigationDestination(keyedBy: Scenes.Preferences.self, in: wallet.id) { _ in
                PreferencesScene(model: viewModelFactory.preferencesScene())
            }
            .navigationDestination(keyedBy: Scenes.Appearance.self, in: wallet.id) { _ in
                AppearanceScene(model: viewModelFactory.appearanceScene())
            }
            .navigationDestination(keyedBy: Scenes.Rewards.self, in: wallet.id) { scene in
                RewardsScene(model: viewModelFactory.rewardsScene(activateCode: scene.code))
            }
            .navigationDestination(keyedBy: Scenes.ChainSettings.self, in: wallet.id) {
                ChainSettingsScene(model: viewModelFactory.chainSettingsScene(chain: $0.chain))
            }
            .navigationDestination(keyedBy: Scenes.Contacts.self, in: wallet.id) { _ in
                ContactsNavigationView(model: viewModelFactory.contactsScene())
            }
        }
    }
}
