// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemProviderRow
import Localization

public struct ProvidersViewModel: SelectableListAdoptable {
    public typealias Item = GemProviderRow

    public var state: StateViewType<SelectableListType<GemProviderRow>>
    public var selectedItems: Set<GemProviderRow>
    public var selectionType: SelectionType

    public init(
        state: StateViewType<SelectableListType<Item>>,
        selectedItems: [GemProviderRow] = [],
        selectionType: SelectionType = .navigationLink,
    ) {
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.selectionType = selectionType
    }

    public var emptyStateTitle: String? {
        Localized.Common.notAvailable
    }

    public var errorTitle: String? {
        Localized.Errors.errorOccurred
    }
}

extension ProvidersViewModel: SelectableListNavigationAdoptable {
    public var title: String {
        Localized.Buy.Providers.title
    }
}
