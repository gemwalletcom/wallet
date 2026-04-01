// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives

struct FiatProvidersViewModel: SelectableListAdoptable {
    typealias Item = FiatQuoteViewModel
    let state: StateViewType<SelectableListType<FiatQuoteViewModel>>
    var selectedItems: Set<FiatQuoteViewModel>
    let selectionType: SelectionType

    init(
        state: StateViewType<SelectableListType<FiatQuoteViewModel>>,
        selectedItems: [FiatQuoteViewModel],
        selectionType: SelectionType,
    ) {
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.selectionType = selectionType
    }
}

extension FiatProvidersViewModel: SelectableListNavigationAdoptable {
    var title: String { Localized.Buy.Providers.title }
    var doneTitle: String { Localized.Common.done }
}
