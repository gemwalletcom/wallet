// Copyright (c). Gem Wallet. All rights reserved.

import UIKit

public enum DeviceSize {
    case small
    case medium
    case large

    @MainActor
    public static var current: DeviceSize {
        switch UIScreen.main.bounds.height {
        case ...667: .small
        case 668 ..< 896: .medium
        case 896 ..< 1000: .large
        case 1000 ... 1200: .medium
        case 1200...: .large
        default: .medium
        }
    }
}
