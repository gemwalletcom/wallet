// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol PushNotificationEnabler: Sendable {
    @discardableResult
    func requestPermissions() async throws -> Bool
}
