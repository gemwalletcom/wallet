// Copyright (c). Gem Wallet. All rights reserved.

import BannerService
import DeviceService
import GemAPI
import NotificationService
import Preferences
import Primitives

public final class OnstartWalletService: Sendable {
    private let deviceService: any DeviceServiceable
    private let bannerSetupService: BannerSetupService
    private let walletConfigurationService: any GemAPIWalletConfigurationService
    private let pushNotificationEnablerService: PushNotificationEnablerService

    public init(
        deviceService: any DeviceServiceable,
        bannerSetupService: BannerSetupService,
        walletConfigurationService: any GemAPIWalletConfigurationService,
        pushNotificationEnablerService: PushNotificationEnablerService,
    ) {
        self.deviceService = deviceService
        self.bannerSetupService = bannerSetupService
        self.walletConfigurationService = walletConfigurationService
        self.pushNotificationEnablerService = pushNotificationEnablerService
    }

    @discardableResult
    public func setup(wallet: Wallet) -> Task<Void, Never> {
        Task {
            try? bannerSetupService.setupWallet(wallet: wallet)
            await syncWalletConfiguration(wallet)
        }
    }

    public func requestPushPermissions() async {
        do {
            let status = try await pushNotificationEnablerService.getNotificationSettingsStatus()

            switch status {
            case .notDetermined:
                let isEnabled = try await pushNotificationEnablerService.requestPermissions()
                if isEnabled {
                    try await deviceService.update()
                }
            case .authorized, .ephemeral, .provisional, .denied:
                return
            @unknown default:
                return
            }
        } catch {
            debugLog("requestPushPermissions error: \(error)")
        }
    }

    private func syncWalletConfiguration(_ wallet: Wallet) async {
        let walletPreferences = WalletPreferences(walletId: wallet.id)
        guard !walletPreferences.completeInitialWalletConfiguration else { return }

        guard let result = try? await walletConfigurationService.getWalletConfiguration(walletId: wallet.id) else { return }

        for account in result.configuration.multiSignatureAccounts {
            try? bannerSetupService.setupAccountMultiSignatureWallet(walletId: wallet.id, chain: account.chain)
        }
        walletPreferences.completeInitialWalletConfiguration = true
    }
}
