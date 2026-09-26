// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemProviderRow
import Localization
import Primitives

struct SwapProvidersViewModel: SelectableListAdoptable {
    typealias Item = GemProviderRow

    var state: StateViewType<SelectableListType<GemProviderRow>>
    var selectedItems: Set<GemProviderRow>
    var selectionType: SelectionType

    init(
        state: StateViewType<SelectableListType<Item>>,
        selectedItems: [GemProviderRow] = [],
        selectionType: SelectionType = .navigationLink,
    ) {
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.selectionType = selectionType
    }

    var emptyStateTitle: String? {
        Localized.Common.notAvailable
    }

    var errorTitle: String? {
        Localized.Errors.errorOccurred
    }
}

extension SwapProvidersViewModel: SelectableListNavigationAdoptable {
    var title: String {
        Localized.Buy.Providers.title
    }
}
