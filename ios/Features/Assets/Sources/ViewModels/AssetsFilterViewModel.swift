// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemChainServiceProtocol
import Components
import Localization
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct AssetsFilterViewModel: Sendable, Equatable {
    private let type: SelectAssetType
    private let chainService: any GemChainServiceProtocol
    var chainsFilter: ChainsFilterViewModel
    var hasBalance: Bool = false

    public init(type: SelectAssetType, model: ChainsFilterViewModel, chainService: any GemChainServiceProtocol) {
        self.chainService = chainService
        self.type = type
        chainsFilter = model
    }

    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.type == rhs.type && lhs.chainsFilter == rhs.chainsFilter && lhs.hasBalance == rhs.hasBalance
    }

    public var isAnyFilterSpecified: Bool {
        chainsFilter.isAnySelected || hasBalance
    }

    var filters: [AssetsRequestFilter] {
        guard isAnyFilterSpecified else { return defaultFilters }

        var result = defaultFilters

        if chainsFilter.isAnySelected {
            result.append(.chains(chainsFilter.selectedChains.map(\.rawValue)))
        }

        if hasBalance, showHasBalanceToggle {
            result.append(.hasBalance)
        }

        return result.unique()
    }

    public var defaultFilters: [AssetsRequestFilter] {
        type.flow().defaultFilters
    }

    var showHasBalanceToggle: Bool {
        type.flow().capabilities.contains(.balanceFilter)
    }

    var title: String {
        Localized.Filter.title
    }

    var clear: String {
        Localized.Filter.clear
    }

    var hasBalanceImageStyle: ListItemImageStyle? {
        .settings(assetImage: .image(Images.Filters.balance))
    }

    var hasBalanceTitle: String {
        Localized.Filter.hasBalance
    }

    var networksModel: NetworkSelectorViewModel {
        NetworkSelectorViewModel(
            state: .data(.plain(chainsFilter.allChains)),
            selectedItems: chainsFilter.selectedChains,
            selectionType: .multiSelection,
            chainService: chainService,
        )
    }
}

// MARK: - Models extensions

extension AssetsRequestFilter {
    var associatedChains: [String] {
        if case let .chains(chains) = self {
            return chains
        }
        return []
    }
}
