// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices

public final class PushNotificationEnablerMock: PushNotificationEnabler, @unchecked Sendable {
    public private(set) var didRequestPermissions = false

    private let granted: Bool

    public init(granted: Bool = true) {
        self.granted = granted
    }

    @discardableResult
    public func requestPermissions() async throws -> Bool {
        didRequestPermissions = true
        return granted
    }
}
