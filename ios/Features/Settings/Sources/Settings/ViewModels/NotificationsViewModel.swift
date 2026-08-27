// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemBannerServiceProtocol
import struct Gemstone.GemBannerKey
import GemstonePrimitives
import GemstoneServices
import Components
import Foundation
import protocol Gemstone.GemDeviceServiceProtocol
import Localization
import Preferences
import Primitives
import Style

@Observable
@MainActor
public final class NotificationsViewModel {
    private let deviceService: any GemDeviceServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol
    private let pushNotificationService: PushNotificationEnablerService
    private let bannerService: any GemBannerServiceProtocol

    var isEnabled: Bool

    public init(
        deviceService: any GemDeviceServiceProtocol,
        bannerService: any GemBannerServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
    ) {
        self.deviceService = deviceService
        self.preferencesService = preferencesService
        pushNotificationService = PushNotificationEnablerService(preferencesService: preferencesService)
        isEnabled = preferencesService.isPushNotificationsEnabled()
        self.bannerService = bannerService
    }

    var title: String {
        Localized.Settings.Notifications.title
    }

    var priceAlertsTitle: String {
        Localized.Settings.PriceAlerts.title
    }

    var priceAlertsImage: AssetImage {
        AssetImage.image(Images.Settings.priceAlerts)
    }
}

// MARK: - Business Logic

extension NotificationsViewModel {
    func enable(isEnabled: Bool) async throws {
        switch isEnabled {
        case true:
            self.isEnabled = try await requestPermissionsOrOpenSettings()
            if isEnabled {
                try await bannerService.close(key: GemBannerKey(walletId: nil, assetId: nil, event: BannerEvent.enableNotifications.json()))
            }
        case false:
            try preferencesService.setPushNotificationsEnabled(enabled: isEnabled)
        }
        try await update()
    }
}

// MARK: - Private

extension NotificationsViewModel {
    private func update() async throws {
        _ = try await deviceService.synchronize()
    }

    private func requestPermissionsOrOpenSettings() async throws -> Bool {
        try await pushNotificationService.requestPermissionsOrOpenSettings()
    }
}
