// Copyright (c). Gem Wallet. All rights reserved.

enum SecurityRow: String, Identifiable {
    case authentication
    case lockPeriod
    case privacyLock
    case hideBalance

    var id: String { rawValue }
}
