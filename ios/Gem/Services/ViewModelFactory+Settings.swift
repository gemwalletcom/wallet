// Copyright (c). Gem Wallet. All rights reserved.

import AppLock
import Foundation
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
import class Gemstone.GemCurrencyService
import class Gemstone.GemNotificationsService
import class Gemstone.GemSettingsService

extension ViewModelFactory {
    @MainActor
    public func notificationsScene() -> NotificationsViewModel {
        NotificationsViewModel(service: GemNotificationsService(device: deviceService, preferences: preferencesService, permissions: notificationPermissions))
    }

    @MainActor
    public func settingsScene() -> SettingsViewModel {
        SettingsViewModel(
            service: GemSettingsService(preferences: preferencesService),
            observablePreferences: observablePreferences,
        )
    }

    @MainActor
    public func appearanceScene() -> AppearanceViewModel {
        AppearanceViewModel(preferences: observablePreferences)
    }

    @MainActor
    public func preferencesScene() -> PreferencesViewModel {
        PreferencesViewModel(
            settings: GemSettingsService(preferences: preferencesService),
            preferences: observablePreferences,
        )
    }

    @MainActor
    public func connectionsScene(
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
    public func aboutUsScene() -> AboutUsViewModel {
        AboutUsViewModel(preferences: observablePreferences, service: appUpdateService)
    }

    @MainActor
    public func chainListSettingsScene() -> ChainListSettingsViewModel {
        ChainListSettingsViewModel(service: chainService)
    }

    @MainActor
    public func serviceStatusScene() -> ServiceStatusViewModel {
        ServiceStatusViewModel(service: serviceStatusService)
    }

    @MainActor
    public func priceAlertsScene() -> PriceAlertsSceneViewModel {
        PriceAlertsSceneViewModel(service: priceAlertService)
    }

    @MainActor
    public func assetPriceAlertsScene(walletId: WalletId, asset: Asset) -> AssetPriceAlertsViewModel {
        AssetPriceAlertsViewModel(service: priceAlertService, walletId: walletId, asset: asset)
    }

    @MainActor
    public func setPriceAlertScene(walletId: WalletId, asset: Asset, onComplete: StringAction) -> SetPriceAlertViewModel {
        SetPriceAlertViewModel(walletId: walletId, asset: asset, service: priceAlertService, onComplete: onComplete)
    }

    @MainActor
    public func inAppNotificationsScene() -> InAppNotificationsViewModel? {
        currentWallet(in: currentWallets()).map { InAppNotificationsViewModel(wallet: $0, service: inAppNotificationService) }
    }

    @MainActor
    public func currencyScene() -> CurrencySceneViewModel {
        CurrencySceneViewModel(
            currencyStorage: observablePreferences,
            service: GemCurrencyService(preferences: preferencesService, prices: priceService, device: deviceService),
        )
    }

    @MainActor
    public func supportChatScene() -> SupportChatSceneViewModel {
        SupportChatSceneViewModel(service: supportService, typing: supportTyping)
    }

    @MainActor
    public func developerScene(walletId: WalletId) -> DeveloperViewModel {
        DeveloperViewModel(walletId: walletId, service: developerService, devicePlatform: devicePlatform)
    }

    @MainActor
    public func lockScene() -> LockSceneViewModel {
        LockSceneViewModel(service: biometryService)
    }

    @MainActor
    public func securityScene() -> SecurityViewModel {
        SecurityViewModel(
            service: biometryService,
            settings: GemSettingsService(preferences: preferencesService),
            preferences: observablePreferences,
        )
    }

    @MainActor
    public func rewardsScene(activateCode: String?) -> RewardsViewModel? {
        let wallets = currentWallets()
        return RewardsViewModel(
            service: rewardsService,
            wallets: wallets,
            currentWallet: currentWallet(in: wallets),
            activateCode: activateCode,
        )
    }

    @MainActor
    public func chainSettingsScene(chain: Chain) -> ChainSettingsSceneViewModel {
        ChainSettingsSceneViewModel(chain: chain, service: gatewayService.chainSettingsService(nodes: nodeService, explorer: explorerService))
    }
}
