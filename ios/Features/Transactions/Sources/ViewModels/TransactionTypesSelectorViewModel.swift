// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemTransactionFilter
import Localization
import Primitives
import Style
import SwiftUI

public struct TransactionTypesSelectorViewModel: SelectableSheetViewable {
    public let selectionType: SelectionType
    public let state: StateViewType<SelectableListType<GemTransactionFilter>>

    public var selectedItems: Set<GemTransactionFilter>

    public init(
        state: StateViewType<SelectableListType<GemTransactionFilter>>,
        selectedItems: [GemTransactionFilter],
        selectionType: SelectionType,
    ) {
        self.selectionType = selectionType
        self.state = state
        self.selectedItems = Set(selectedItems)
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

    public var confirmButtonTitle: String {
        Localized.Transfer.confirm
    }
}
