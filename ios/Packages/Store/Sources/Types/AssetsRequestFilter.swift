// Copyright (c). Gem Wallet. All rights reserved.

public enum AssetsRequestFilter {
    case search(String, hasPriorityAssets: Bool)
    case enabled
    case buyable
    case sellable
    case swappable
    case stakeable
    case enabledBalance
    case disabledBalance
    case hasBalance
    case hasAvailableBalance
    // include all assets of these chains
    case chains([String])
    case chainsOrAssets([String], [String])

    /// AssetData with empty properties
    case priceAlerts
}

extension AssetsRequestFilter: Equatable, Hashable {}
extension AssetsRequestFilter: Sendable {}
