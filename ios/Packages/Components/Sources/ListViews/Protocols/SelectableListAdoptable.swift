// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public protocol SelectableListAdoptable {
    associatedtype Item: Hashable & Identifiable & Sendable
    var selectionType: SelectionType { get }

    var state: StateViewType<SelectableListType<Item>> { get }
    var selectedItems: Set<Item> { get set }

    var emptyStateTitle: String? { get }
    var errorTitle: String? { get }
    var search: ListSearch<Item>? { get }

    mutating func reset()
    mutating func toggle(item: Item)
}

public extension SelectableListAdoptable {
    var items: [Item] {
        state.value?.items ?? []
    }

    var shouldResetOnToggle: Bool {
        switch selectionType {
        case .multiSelection: false
        case .navigationLink, .checkmark: true
        }
    }

    var emptyStateTitle: String? {
        nil
    }

    var errorTitle: String? {
        nil
    }

    var search: ListSearch<Item>? {
        nil
    }

    var filteredState: StateViewType<SelectableListType<Item>> {
        guard let search else { return state }
        return state.map(search.filtered)
    }

    mutating func toggle(item: Item) {
        if shouldResetOnToggle {
            reset()
        }

        if selectedItems.contains(item) {
            selectedItems.remove(item)
        } else {
            selectedItems.insert(item)
        }
    }

    mutating func reset() {
        selectedItems = []
    }

    mutating func select(item: Item) -> [Item]? {
        toggle(item: item)

        return switch selectionType {
        case .multiSelection: nil
        case .navigationLink, .checkmark: Array(selectedItems)
        }
    }
}
