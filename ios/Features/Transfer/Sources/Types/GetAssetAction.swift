// Copyright (c). Gem Wallet. All rights reserved.

public enum GetAssetAction: String, Identifiable, Sendable {
    case buy
    case swap
    case receive

    public var id: String {
        rawValue
    }
}
