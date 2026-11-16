// Copyright (c). Gem Wallet. All rights reserved.

import Components
import class Gemstone.GemChainService
import struct Gemstone.GemLeverageSelection
import struct Gemstone.GemProviderRow
import struct Gemstone.GemWalletRow
import struct Gemstone.GemWalletSection
import GemstonePrimitives
import Localization
import Primitives

public struct SelectableListViewModel<Item: Hashable & Identifiable & Sendable>: SelectableSheetViewable, SelectableListNavigationAdoptable {
    public let title: String
    public let state: StateViewType<SelectableListType<Item>>
    public var selectedItems: Set<Item>
    public let selectionType: SelectionType
    public let emptyStateTitle: String?
    public let errorTitle: String?
    public let search: ListSearch<Item>?

    public init(
        title: String,
        state: StateViewType<SelectableListType<Item>>,
        selectedItems: [Item] = [],
        selectionType: SelectionType = .navigationLink,
        emptyStateTitle: String? = nil,
        errorTitle: String? = nil,
        search: ListSearch<Item>? = nil,
    ) {
        self.title = title
        self.state = state
        self.selectedItems = Set(selectedItems)
        self.selectionType = selectionType
        self.emptyStateTitle = emptyStateTitle
        self.errorTitle = errorTitle
        self.search = search
    }
}

public extension SelectableListViewModel where Item == GemProviderRow {
    init(providers state: StateViewType<SelectableListType<GemProviderRow>>) {
        self.init(
            title: Localized.Buy.Providers.title,
            state: state,
            emptyStateTitle: Localized.Common.notAvailable,
            errorTitle: Localized.Errors.errorOccurred,
        )
    }
}

public extension SelectableListViewModel where Item == Chain {
    init(
        networks state: StateViewType<SelectableListType<Chain>>,
        selectedItems: [Chain] = [],
        selectionType: SelectionType = .navigationLink,
        title: String = Localized.Settings.Networks.title,
    ) {
        self.init(
            title: title,
            state: state,
            selectedItems: selectedItems,
            selectionType: selectionType,
            search: ListSearch(
                filter: Self.filter(chains:query:),
                emptyContent: EmptyStateViewModel(kind: .searchNetworks),
            ),
        )
    }

    private static func filter(chains: [Chain], query: String) -> [Chain] {
        GemChainService.shared.chainRows(chains: chains.map(\.rawValue), query: query).map { Chain(core: $0.chain) }
    }
}

public extension SelectableListViewModel where Item == LeverageOption {
    init(leverage selection: GemLeverageSelection) {
        self.init(
            title: Localized.Perpetual.leverage,
            state: .data(.plain(selection.options.map(LeverageOption.init(option:)))),
            selectedItems: [LeverageOption(option: selection.selected)],
            selectionType: .checkmark,
        )
    }
}

public extension SelectableListViewModel where Item == GemWalletRow {
    init(sections: [GemWalletSection], selectedRow: GemWalletRow) {
        self.init(
            title: Localized.Wallets.title,
            state: .data(.section(sections.map { ListSection(id: String(describing: $0.kind), title: $0.kind.title, image: $0.kind.image, values: $0.rows) })),
            selectedItems: [selectedRow],
            selectionType: .checkmark,
        )
    }
}
