// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

public struct TransactionTypesSelectorViewModel: SelectableSheetViewable {
    public var items: [TransactionFilterType] {
        state.value?.items ?? []
    }

    public let selectionType: SelectionType
    public let state: StateViewType<SelectableListType<TransactionFilterType>>

    public var selectedItems: Set<TransactionFilterType>

    public init(
        state: StateViewType<SelectableListType<TransactionFilterType>>,
        selectedItems: [TransactionFilterType],
        selectionType: SelectionType,
    ) {
        self.selectionType = selectionType
        self.state = state
        self.selectedItems = Set(selectedItems)
    }

    public var isSearchable: Bool {
        false
    }

    public var title: String {
        Localized.Filter.types
    }

    public var cancelButtonTitle: String {
        Localized.Common.cancel
    }

    public var clearButtonTitle: String {
        Localized.Filter.clear
    }

    public var doneButtonTitle: String {
        Localized.Common.done
    }

    public var emptyCotentModel: (any EmptyContentViewable)? {
        .none
    }

    public var confirmButtonTitle: String {
        Localized.Transfer.confirm
    }
}
