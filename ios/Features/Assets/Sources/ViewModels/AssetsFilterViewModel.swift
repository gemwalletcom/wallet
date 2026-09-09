// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import struct Gemstone.GemSelectAssetFlow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct AssetsFilterViewModel: Sendable, Equatable {
    private let flow: GemSelectAssetFlow
    var chainsFilter: ChainsFilterViewModel
    var hasBalance: Bool = false

    public init(flow: GemSelectAssetFlow, model: ChainsFilterViewModel) {
        self.flow = flow
        chainsFilter = model
    }

    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.flow == rhs.flow && lhs.chainsFilter == rhs.chainsFilter && lhs.hasBalance == rhs.hasBalance
    }

    public var isAnyFilterSpecified: Bool {
        chainsFilter.isAnySelected || hasBalance
    }

    var filters: [AssetsRequestFilter] {
        guard isAnyFilterSpecified else { return flow.requestFilters }

        var result = flow.requestFilters

        if chainsFilter.isAnySelected {
            result.append(.chains(chainsFilter.selectedChains.map(\.rawValue)))
        }

        if hasBalance, showHasBalanceToggle {
            result.append(.hasBalance)
        }

        return result.unique()
    }

    var showHasBalanceToggle: Bool {
        flow.balanceFilter
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
