// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAssetAction
import enum Gemstone.GemAssetFilter
import func Gemstone.assetActionFilters

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

public extension [AssetsRequestFilter] {
    static func filters(for action: GemAssetAction) -> [AssetsRequestFilter] {
        assetActionFilters(action: action).map { filter in
            switch filter {
            case .enabled: .enabled
            case .buyable: .buyable
            case .sellable: .sellable
            case .swappable: .swappable
            case .hasBalance: .hasBalance
            case .hasAvailableBalance: .hasAvailableBalance
            }
        }
    }
}
