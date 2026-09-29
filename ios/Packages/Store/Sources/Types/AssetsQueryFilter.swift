// Copyright (c). Gem Wallet. All rights reserved.

public enum AssetsQueryFilter {
    case search(String, hasPriorityAssets: Bool)
    case enabled
    case buyable
    case sellable
    case swappable
    case enabledBalance
    case disabledBalance
    case hasBalance
    case hasAvailableBalance
    case chains([String])
    case chainsOrAssets([String], [String])
}

extension AssetsQueryFilter: Equatable, Hashable {}
extension AssetsQueryFilter: Sendable {}
