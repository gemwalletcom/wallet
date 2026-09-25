// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSelectAssetFlow
import GemstonePrimitives
import GemstoneServices
import Localization
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

    public var isAnyFilterSpecified: Bool {
        chainsFilter.isAnySelected || hasBalance
    }

    var filters: [AssetsQueryFilter] {
        flow.appliedFilters(chains: chainsFilter.selectedChains.map(\.rawValue), hasBalance: hasBalance).map { $0.map() }
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
