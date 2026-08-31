// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNotificationPermissions

public final class NotificationPermissionsMock: GemNotificationPermissions, @unchecked Sendable {
    public private(set) var requestCount = 0
    private let granted: Bool

    public init(granted: Bool = true) {
        self.granted = granted
    }

    public func requestPermissionsOrOpenSettings() async throws -> Bool {
        requestCount += 1
        return granted
    }
}
