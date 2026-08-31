// Copyright (c). Gem Wallet. All rights reserved.

import UserNotifications

extension UNAuthorizationStatus {
    var isAuthorized: Bool {
        switch self {
        case .authorized, .ephemeral, .provisional: true
        case .denied, .notDetermined: false
        @unknown default: false
        }
    }
}
