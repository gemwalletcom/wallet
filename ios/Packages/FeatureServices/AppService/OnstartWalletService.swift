// Copyright (c). Gem Wallet. All rights reserved.

import BannerService
import DeviceService
import protocol Gemstone.GemWalletConfigurationServiceProtocol
import NotificationService
import Primitives

public final class OnstartWalletService: Sendable {
    private let deviceService: any DeviceServiceable
    private let bannerSetupService: BannerSetupService
    private let walletConfigurationService: any GemWalletConfigurationServiceProtocol
    private let pushNotificationEnablerService: PushNotificationEnablerService

    public init(
        deviceService: any DeviceServiceable,
        bannerSetupService: BannerSetupService,
        walletConfigurationService: any GemWalletConfigurationServiceProtocol,
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
            try? await walletConfigurationService.sync(walletId: wallet.id.id)
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
}
