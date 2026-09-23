// Copyright (c). Gem Wallet. All rights reserved.

import AppLock
import Foundation
import class Gemstone.GemCurrencyService
import class Gemstone.GemNotificationsService
import class Gemstone.GemSettingsService
import GemstonePrimitives
import GemstoneServices
import InAppNotifications
import PriceAlerts
import Primitives
import PrimitivesComponents
import Settings
import Store
import Support
import SwiftUI
import WalletConnector
import WalletConnectorService

public extension ViewModelFactory {
    @MainActor
    func notificationsScene() -> NotificationsViewModel {
        NotificationsViewModel(service: GemNotificationsService(device: deviceService, preferences: preferencesService, permissions: notificationPermissions))
    }

    @MainActor
    func settingsScene() -> SettingsViewModel {
        SettingsViewModel(
            service: GemSettingsService(preferences: preferencesService),
            notifications: GemNotificationsService(device: deviceService, preferences: preferencesService, permissions: notificationPermissions),
            observablePreferences: observablePreferences,
        )
    }

    @MainActor
    func appearanceScene() -> AppearanceViewModel {
        AppearanceViewModel(preferences: observablePreferences)
    }

    @MainActor
    func preferencesScene() -> PreferencesViewModel {
        PreferencesViewModel(
            settings: GemSettingsService(preferences: preferencesService),
            preferences: observablePreferences,
        )
    }

    @MainActor
    func connectionsScene(
        connector: any WalletConnectorServiceable,
        walletConnectorPresenter: WalletConnectorPresenter,
    ) -> ConnectionsViewModel {
        ConnectionsViewModel(
            connector: connector,
            service: walletConnectService,
            walletConnectorPresenter: walletConnectorPresenter,
        )
    }

    @MainActor
    func aboutUsScene() -> AboutUsViewModel {
        AboutUsViewModel(preferences: observablePreferences, service: appUpdateService)
    }

    @MainActor
    func chainListSettingsScene() -> ChainListSettingsViewModel {
        ChainListSettingsViewModel(service: gatewayService.chainSettingsService(nodes: nodeService, explorer: explorerService))
    }

    @MainActor
    func serviceStatusScene() -> ServiceStatusViewModel {
        ServiceStatusViewModel(service: serviceStatusService)
    }

    @MainActor
    func priceAlertsScene() -> PriceAlertsSceneViewModel {
        PriceAlertsSceneViewModel(service: priceAlertService)
    }

    @MainActor
    func assetPriceAlertsScene(walletId: WalletId, asset: Asset) -> AssetPriceAlertsViewModel {
        AssetPriceAlertsViewModel(service: priceAlertService, walletId: walletId, asset: asset)
    }

    @MainActor
    func setPriceAlertScene(walletId: WalletId, asset: Asset, onComplete: StringAction) -> SetPriceAlertViewModel {
        SetPriceAlertViewModel(walletId: walletId, asset: asset, service: priceAlertService, onComplete: onComplete)
    }

    @MainActor
    func inAppNotificationsScene() -> InAppNotificationsViewModel? {
        currentWallet(in: currentWallets()).map {
            InAppNotificationsViewModel(wallet: $0, service: inAppNotificationService) { action in
                Task { await AppResolver.main.services.navigationRouter.open(action: action) }
            }
        }
    }

    @MainActor
    func currencyScene() -> CurrencySceneViewModel {
        CurrencySceneViewModel(
            preferences: observablePreferences,
            service: GemCurrencyService(preferences: preferencesService, prices: priceService),
        )
    }

    @MainActor
    func supportChatScene() -> SupportChatSceneViewModel {
        SupportChatSceneViewModel(service: supportService, typing: supportTyping)
    }

    @MainActor
    func developerScene(walletId: WalletId) -> DeveloperViewModel {
        DeveloperViewModel(walletId: walletId, service: developerService, devicePlatform: devicePlatform)
    }

    @MainActor
    func lockScene() -> LockSceneViewModel {
        LockSceneViewModel(service: biometryService)
    }

    @MainActor
    func securityScene() -> SecurityViewModel {
        SecurityViewModel(
            service: biometryService,
            settings: GemSettingsService(preferences: preferencesService),
            preferences: observablePreferences,
        )
    }

    @MainActor
    func rewardsScene(activateCode: String?) -> RewardsViewModel? {
        let wallets = currentWallets()
        return RewardsViewModel(
            service: rewardsService,
            wallets: wallets,
            currentWallet: currentWallet(in: wallets),
            activateCode: activateCode,
        )
    }

    @MainActor
    func chainSettingsScene(chain: Chain) -> ChainSettingsSceneViewModel {
        ChainSettingsSceneViewModel(chain: chain, service: gatewayService.chainSettingsService(nodes: nodeService, explorer: explorerService))
    }
}
