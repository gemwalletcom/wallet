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
            emptyContent: EmptyContentTypeViewModel(type: EmptyContentType(.searchNetworks)),
        )
    }

    private func filter(chains: [Chain], query: String) -> [Chain] {
        let matching = Set(GemChainService.shared.getMatchingChains(chains: chains.map(\.rawValue), query: query))
        return chains.filter { matching.contains($0.rawValue) }
    }
}
