// Copyright (c). Gem Wallet. All rights reserved.

public enum GetNetworkFeeAssetAction: String, Identifiable, Sendable {
    case buy
    case swap
    case receive

    public var id: String { rawValue }
}
