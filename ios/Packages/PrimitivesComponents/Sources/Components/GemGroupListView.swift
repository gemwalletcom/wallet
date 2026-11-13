// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemGroup
import struct Gemstone.GemGroupList
import enum Gemstone.GemListItem
import SwiftUI

public struct GemGroupListView: View {
    private let list: GemGroupList?
    private let onSelect: (GemListItem) -> Void

    public init(list: GemGroupList?, onSelect: @escaping (GemListItem) -> Void) {
        self.list = list
        self.onSelect = onSelect
    }

    public var body: some View {
        List {
            ForEach(keyedGroups) { group in
                if let footer = group.value.footer.text {
                    Section {
                        items(group.value)
                    } header: {
                        header(group.value)
                    } footer: {
                        Text(footer)
                    }
                } else {
                    Section {
                        items(group.value)
                    } header: {
                        header(group.value)
                    }
                }
            }
            if case let .error(error) = list?.phase {
                Section {
                    GemListRowView(row: .error(error: error))
                }
            }
        }
        .overlay {
            if case let .empty(state) = list?.phase {
                EmptyContentView(model: EmptyStateViewModel(state: state))
            }
        }
    }

    private func items(_ group: GemGroup) -> some View {
        ForEach(keyedItems(group)) { item in
            GemListItemView(item: item.value, onSelect: onSelect)
        }
    }

    @ViewBuilder
    private func header(_ group: GemGroup) -> some View {
        if let title = group.title.text {
            Text(title)
        }
    }

    private var keyedGroups: [Keyed<GemGroup>] {
        (list?.groups ?? []).enumerated().map { index, group in
            Keyed(id: group.title == .none ? "\(index)" : "\(group.title)", value: group)
        }
    }

    private func keyedItems(_ group: GemGroup) -> [Keyed<GemListItem>] {
        group.items.enumerated().map { index, item in
            Keyed(id: item.identity ?? "\(index)", value: item)
        }
    }
}

private struct Keyed<Value>: Identifiable {
    let id: String
    let value: Value
}

struct GemListItemView: View {
    let item: GemListItem
    let onSelect: (GemListItem) -> Void

    var body: some View {
        switch item {
        case let .currency(row):
            ListItemSelectionView(
                title: row.title,
                value: row.currency,
                selection: row.isSelected ? row.currency : nil,
            ) { _ in
                onSelect(item)
            }
            .accessibilityIdentifier(row.currency.toPrimitives().rawValue)
        }
    }
}

extension GemListItem {
    var identity: String? {
        switch self {
        case let .currency(row): row.currency.toPrimitives().rawValue
        }
    }
}
