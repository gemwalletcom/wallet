// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import Contacts
import GemstoneServices
import InAppNotifications
import MarketInsight
import Preferences
import PriceAlerts
import Primitives
import PrimitivesComponents
import Settings
import Store
import Support
import SwiftUI
import WalletConnector

struct SettingsNavigationView: View {
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationHandler) private var navigationHandler
    @Environment(\.transactionsService) private var transactionsService
    @Environment(\.assetStore) private var assetStore
    @Environment(\.stakeStore) private var stakeStore
    @Environment(\.transactionStore) private var transactionStore
    @Environment(\.priceStore) private var priceStore
    @Environment(\.explorerService) private var explorerService
    @Environment(\.bannerService) private var bannerService
    @Environment(\.bannerStore) private var bannerStore
    @Environment(\.connectionsService) private var connectionsService
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.walletSessionService) private var walletSessionService
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.priceService) private var priceService
    @Environment(\.chartService) private var chartService
    @Environment(\.nodeService) private var nodeService
    @Environment(\.serviceStatusService) private var serviceStatusService
    @Environment(\.chainServiceFactory) private var chainServiceFactory
    @Environment(\.observablePreferences) private var observablePreferences
    @Environment(\.appUpdateService) private var appUpdateService
    @Environment(\.perpetualService) private var perpetualService
    @Environment(\.walletConnectorManager) private var walletConnectorManager
    @Environment(\.rewardsService) private var rewardsService
    @Environment(\.inAppNotificationService) private var inAppNotificationService
    @Environment(\.contactService) private var contactService
    @Environment(\.nameService) private var nameService
    @Environment(\.supportService) private var supportService
    @Environment(\.walletPreferencesService) private var walletPreferencesService
    @Environment(\.supportTypingState) private var supportTypingState
    @Environment(\.navigationPresenter) private var presenter

    @State private var currencyModel: CurrencySceneViewModel

    let walletId: WalletId
    private let deviceService: any GemDeviceServiceProtocol
    @Binding var isPresentingSupport: Bool

    init(
        walletId: WalletId,
        preferences: Preferences = .standard,
        priceService: any GemPriceServiceProtocol,
        deviceService: any GemDeviceServiceProtocol,
        isPresentingSupport: Binding<Bool>,
    ) {
        self.walletId = walletId
        self.deviceService = deviceService
        _isPresentingSupport = isPresentingSupport
        _currencyModel = State(
            initialValue: CurrencySceneViewModel(
                currencyStorage: preferences,
                priceService: priceService,
                deviceService: deviceService,
            ),
        )
    }

    var body: some View {
        SettingsScene(
            model: SettingsViewModel(
                walletId: walletId,
                walletSessionService: walletSessionService,
                observablePreferences: observablePreferences,
            ),
            isPresentingWallets: presenter.isPresentingWallets,
            isPresentingSupport: $isPresentingSupport,
        )
        .navigationBarTitleDisplayMode(.inline)
        .navigationDestination(for: Scenes.Security.self) { _ in
            SecurityScene(model: SecurityViewModel())
        }
        .navigationDestination(for: Scenes.Notifications.self) { _ in
            NotificationsScene(
                model: NotificationsViewModel(
                    deviceService: deviceService,
                    bannerService: bannerService,
                ),
            )
        }
        .navigationDestination(for: Scenes.PriceAlerts.self) { _ in
            PriceAlertsNavigationView(
                model: PriceAlertsSceneViewModel(priceAlertService: priceAlertService),
            )
        }
        .navigationDestination(for: Scenes.AssetPriceAlert.self) {
            AssetPriceAlertsScene(
                model: AssetPriceAlertsViewModel(
                    priceAlertService: priceAlertService,
                    walletId: walletId,
                    asset: $0.asset,
                ),
            )
        }
        .navigationDestination(for: Scenes.Price.self) { scene in
            ChartScene(
                model: ChartSceneViewModel(
                    explorerService: explorerService,
                    service: chartService,
                    priceStore: priceStore,
                    assetModel: AssetViewModel(asset: scene.asset),
                    priceAlertService: priceAlertService,
                    walletId: walletId,
                    onSetPriceAlert: { _ in },
                ),
            )
        }
        .navigationDestination(for: Scenes.Chains.self) { _ in
            ChainListSettingsScene()
        }
        .navigationDestination(for: Scenes.ServiceStatus.self) { _ in
            ServiceStatusScene(model: ServiceStatusViewModel(serviceStatusService: serviceStatusService))
        }
        .navigationDestination(for: Scenes.AboutUs.self) { _ in
            AboutUsScene(
                model: AboutUsViewModel(
                    preferences: observablePreferences,
                    appUpdateService: appUpdateService,
                ),
            )
        }
        .navigationDestination(for: Scenes.WalletConnect.self) { _ in
            ConnectionsScene(
                model: ConnectionsViewModel(
                    service: connectionsService,
                    walletConnectorPresenter: walletConnectorManager.presenter,
                ),
            )
        }
        .navigationDestination(for: Scenes.Developer.self) { _ in
            DeveloperScene(model: DeveloperViewModel(
                walletId: walletId,
                transactionStore: transactionStore,
                assetStore: assetStore,
                stakeStore: stakeStore,
                bannerStore: bannerStore,
                priceStore: priceStore,
                perpetualService: perpetualService,
                walletPreferencesService: walletPreferencesService,
            ))
        }
        .navigationDestination(for: Scenes.DeveloperPayments.self) { _ in
            DeveloperPaymentsScene { payload in
                Task { await navigationHandler.handle(code: payload) }
            }
        }
        .navigationDestination(for: Scenes.InAppNotifications.self) { _ in
            if let wallet = walletSessionService.currentWallet {
                InAppNotificationsScene(
                    model: InAppNotificationsViewModel(
                        wallet: wallet,
                        notificationService: inAppNotificationService,
                    ),
                )
            }
        }
        .navigationDestination(for: Scenes.Currency.self) { _ in
            CurrencyScene(model: currencyModel)
        }
        .navigationDestination(for: Scenes.Preferences.self) { _ in
            PreferencesScene(model: PreferencesViewModel(currencyModel: currencyModel))
        }
        .navigationDestination(for: Scenes.Appearance.self) { _ in
            AppearanceScene(model: AppearanceViewModel())
        }
        .navigationDestination(for: Scenes.Referral.self) { scene in
            let wallets = walletSessionService.wallets.filter { $0.type == .multicoin }
            if let wallet = wallets.first(where: { $0.id == walletSessionService.currentWallet?.id }) ?? wallets.first {
                RewardsScene(
                    model: RewardsViewModel(
                        rewardsService: rewardsService,
                        wallet: wallet,
                        wallets: wallets,
                        activateCode: scene.code,
                    ),
                )
            }
        }
        .navigationDestination(for: Scenes.ChainSettings.self) {
            ChainSettingsScene(
                model: ChainSettingsSceneViewModel(
                    nodeService: nodeService,
                    chainServiceFactory: chainServiceFactory,
                    explorerService: explorerService,
                    chain: $0.chain,
                ),
            )
        }
        .navigationDestination(for: Scenes.Contacts.self) { _ in
            ContactsNavigationView(
                model: ContactsViewModel(service: contactService, nameService: nameService),
            )
        }
        .sheet(isPresented: $isPresentingSupport) {
            NavigationStack {
                SupportChatScene(model: SupportChatSceneViewModel(service: supportService, typing: supportTypingState))
                    .toolbarDismissItem(type: .close, placement: .topBarLeading)
            }
            .environment(\.openURL, OpenURLAction { url in
                guard navigationHandler.open(url: url) else { return .systemAction }
                isPresentingSupport = false
                return .handled
            })
        }
    }
}

extension Preferences: @retroactive CurrencyStorable {}
