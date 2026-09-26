// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemProviderRow
import Localization
import Primitives

struct FiatProvidersViewModel: SelectableListAdoptable {
    typealias Item = GemProviderRow
    let state: StateViewType<SelectableListType<GemProviderRow>>
    var selectedItems: Set<GemProviderRow>
    let selectionType: SelectionType

    init(
        state: StateViewType<SelectableListType<GemProviderRow>>,
        selectedItems: [GemProviderRow] = [],
        selectionType: SelectionType = .navigationLink,
    ) {
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.selectionType = selectionType
    }
}

extension FiatProvidersViewModel: SelectableListNavigationAdoptable {
    var title: String {
        Localized.Buy.Providers.title
    }
}
