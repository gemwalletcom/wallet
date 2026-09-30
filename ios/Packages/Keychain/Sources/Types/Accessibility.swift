// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum Accessibility: Sendable {
    case afterFirstUnlock
    case whenUnlockedThisDeviceOnly

    var rawValue: String {
        switch self {
        case .afterFirstUnlock: String(kSecAttrAccessibleAfterFirstUnlock)
        case .whenUnlockedThisDeviceOnly: String(kSecAttrAccessibleWhenUnlockedThisDeviceOnly)
        }
    }
}
