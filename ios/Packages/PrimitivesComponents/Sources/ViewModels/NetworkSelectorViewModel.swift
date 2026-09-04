// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemChainService
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct NetworkSelectorViewModel: SelectableSheetViewable {
    public var selectionType: SelectionType

    public let state: StateViewType<SelectableListType<Chain>>

    public var selectedItems: Set<Chain>
    public private(set) var search: ListSearch<Chain>?

    public let title: String


    public init(
        state: StateViewType<SelectableListType<Chain>>,
        selectedItems: [Chain] = [],
        selectionType: SelectionType = .navigationLink,
        title: String = Localized.Settings.Networks.title,
    ) {
        self.selectionType = selectionType
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.title = title
        search = ListSearch(
            filter: filter(chain:query:),
            emptyContent: EmptyContentTypeViewModel(type: .search(type: EmptyContentType.SearchType.networks)),
        )
    }

    public var cancelButtonTitle: String { Localized.Common.cancel }
    public var clearButtonTitle: String { Localized.Filter.clear }
    public var doneButtonTitle: String { Localized.Common.done }
    public var confirmButtonTitle: String { Localized.Transfer.confirm }

    private func filter(chain: Chain, query: String) -> Bool {
        !GemChainService.shared.getMatchingChains(chains: [chain.rawValue], query: query).isEmpty
    }
}
