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
            filter: filter(chains:query:),
            emptyContent: EmptyStateViewModel(kind: .searchNetworks),
        )
    }

    private func filter(chains: [Chain], query: String) -> [Chain] {
        GemChainService.shared.chainRows(chains: chains.map(\.rawValue), query: query).map { Chain(core: $0.chain) }
    }
}
