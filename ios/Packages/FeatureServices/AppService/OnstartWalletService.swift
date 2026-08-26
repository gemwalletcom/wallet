// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemBannerServiceProtocol
import protocol Gemstone.GemWalletConfigurationServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Primitives

public final class OnstartWalletService: Sendable {
    private let deviceService: any DeviceServiceable
    private let bannerService: any GemBannerServiceProtocol
    private let walletConfigurationService: any GemWalletConfigurationServiceProtocol
    private let pushNotificationEnablerService: PushNotificationEnablerService

    public init(
        deviceService: any DeviceServiceable,
        bannerService: any GemBannerServiceProtocol,
        walletConfigurationService: any GemWalletConfigurationServiceProtocol,
        pushNotificationEnablerService: PushNotificationEnablerService,
    ) {
        self.deviceService = deviceService
        self.bannerService = bannerService
        self.walletConfigurationService = walletConfigurationService
        self.pushNotificationEnablerService = pushNotificationEnablerService
    }

    @discardableResult
    public func setup(wallet: Wallet) -> Task<Void, Never> {
        Task {
            try? await bannerService.setupWallet(wallet: wallet.json())
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
