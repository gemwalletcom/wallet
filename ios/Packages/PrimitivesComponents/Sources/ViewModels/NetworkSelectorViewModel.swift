// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemChainService
import protocol Gemstone.GemChainServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct NetworkSelectorViewModel: SelectableSheetViewable {
    public var selectionType: SelectionType

    public let state: StateViewType<SelectableListType<Chain>>

    public var selectedItems: Set<Chain>

    private let chainService: any GemChainServiceProtocol

    public init(
        state: StateViewType<SelectableListType<Chain>>,
        selectedItems: [Chain] = [],
        selectionType: SelectionType = .navigationLink,
        chainService: any GemChainServiceProtocol,
    ) {
        self.chainService = chainService
        self.selectionType = selectionType
        self.state = state
        self.selectedItems = Set(selectedItems)
    }

    public var title: String {
        Localized.Settings.Networks.title
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

    public var search: ListSearch<Chain>? {
        ListSearch(
            filter: filter(chain:query:),
            emptyContent: EmptyContentTypeViewModel(type: .search(type: EmptyContentType.SearchType.networks)),
        )
    }

    private func filter(chain: Chain, query: String) -> Bool {
        !chainService.getMatchingChains(chains: [chain.rawValue], query: query).isEmpty
    }
}
