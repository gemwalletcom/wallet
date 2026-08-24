// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import PrimitivesComponents

struct PaymentQuotesViewModel: SelectableListAdoptable {
    typealias Item = PaymentQuoteItem

    var state: StateViewType<SelectableListType<PaymentQuoteItem>>
    var selectedItems: Set<PaymentQuoteItem>
    var selectionType: SelectionType

    init(
        state: StateViewType<SelectableListType<Item>>,
        selectedItems: [PaymentQuoteItem],
        selectionType: SelectionType,
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

// MARK: - SelectableListNavigationAdoptable

extension PaymentQuotesViewModel: SelectableListNavigationAdoptable {
    var title: String {
        Localized.Transfer.payWith
    }

    var doneTitle: String {
        Localized.Common.done
    }
}
