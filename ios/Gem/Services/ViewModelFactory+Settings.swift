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
import Rewards
import Settings
import Store
import Support
import SwiftUI
import WalletConnector
import WalletConnectorService

public extension ViewModelFactory {
    @MainActor
    func notificationsScene() -> NotificationsSceneViewModel {
        NotificationsSceneViewModel(service: GemNotificationsService(device: deviceService, preferences: preferencesService, permissions: notificationPermissions))
    }

    @MainActor
    func settingsScene() -> SettingsSceneViewModel {
        SettingsSceneViewModel(
            service: GemSettingsService(preferences: preferencesService),
            observablePreferences: observablePreferences,
        )
    }

    @MainActor
    func appearanceScene() -> AppearanceSceneViewModel {
        AppearanceSceneViewModel(preferences: observablePreferences)
    }

    @MainActor
    func preferencesScene() -> PreferencesSceneViewModel {
        PreferencesSceneViewModel(
            settings: GemSettingsService(preferences: preferencesService),
            preferences: observablePreferences,
        )
    }

    @MainActor
    func connectionsScene(
        connector: any WalletConnectorServiceable,
        walletConnectorPresenter: WalletConnectorPresenter,
    ) -> ConnectionsSceneViewModel {
        ConnectionsSceneViewModel(
            connector: connector,
            service: walletConnectService,
            walletConnectorPresenter: walletConnectorPresenter,
        )
    }

    @MainActor
    func aboutUsScene() -> AboutUsSceneViewModel {
        AboutUsSceneViewModel(preferences: observablePreferences, service: appUpdateService)
    }

    @MainActor
    func chainListSettingsScene() -> ChainListSettingsSceneViewModel {
        ChainListSettingsSceneViewModel(service: gatewayService.chainSettingsService(nodes: nodeService, explorer: explorerService))
    }

    @MainActor
    func serviceStatusScene() -> ServiceStatusSceneViewModel {
        ServiceStatusSceneViewModel(service: serviceStatusService)
    }

    @MainActor
    func priceAlertsScene() -> PriceAlertsSceneViewModel {
        PriceAlertsSceneViewModel(service: priceAlertService)
    }

    @MainActor
    func assetPriceAlertsScene(walletId: WalletId, asset: Asset) -> AssetPriceAlertsSceneViewModel {
        AssetPriceAlertsSceneViewModel(service: priceAlertService, walletId: walletId, asset: asset)
    }

    @MainActor
    func setPriceAlertScene(walletId: WalletId, asset: Asset, onComplete: StringAction) -> SetPriceAlertSceneViewModel {
        SetPriceAlertSceneViewModel(walletId: walletId, asset: asset, service: priceAlertService, onComplete: onComplete)
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
            service: GemCurrencyService(prices: priceService),
        )
    }

    @MainActor
    func supportChatScene() -> SupportChatSceneViewModel {
        SupportChatSceneViewModel(
            service: supportService,
            notifications: GemNotificationsService(device: deviceService, preferences: preferencesService, permissions: notificationPermissions),
            typing: supportTyping,
        )
    }

    @MainActor
    func developerScene(walletId: WalletId) -> DeveloperSceneViewModel {
        DeveloperSceneViewModel(walletId: walletId, service: developerService, devicePlatform: devicePlatform)
    }

    @MainActor
    func lockScene() -> LockSceneViewModel {
        LockSceneViewModel(service: biometryService)
    }

    @MainActor
    func securityScene() -> SecuritySceneViewModel {
        SecuritySceneViewModel(
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
