// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNotificationPermissions

public final class GemstoneNotificationPermissions: GemNotificationPermissions, Sendable {
    private let service: PushNotificationEnablerService

    public init(service: PushNotificationEnablerService) {
        self.service = service
    }

    public func isAvailable() -> Bool {
        true
    }

    public func isGranted() async -> Bool {
        await service.getNotificationSettingsStatus().isAuthorized
    }

    public func requestPermissionsOrOpenSettings() async throws -> Bool {
        try await service.requestPermissionsOrOpenSettings()
    }
}
