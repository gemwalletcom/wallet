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
            GemGroupSections(groups: list?.groups ?? []) { item in
                GemListItemView(item: item, onSelect: onSelect)
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
}

public struct GemGroupSections<Item: View>: View {
    private let groups: [GemGroup]
    private let item: (GemListItem) -> Item

    public init(groups: [GemGroup], @ViewBuilder item: @escaping (GemListItem) -> Item) {
        self.groups = groups
        self.item = item
    }

    public var body: some View {
        ForEach(groups) { group in
            if let footer = group.footer.text {
                Section {
                    items(group)
                } header: {
                    header(group)
                } footer: {
                    Text(footer)
                }
            } else {
                Section {
                    items(group)
                } header: {
                    header(group)
                }
            }
        }
    }

    private func items(_ group: GemGroup) -> some View {
        ForEach(keyedItems(group)) { keyed in
            item(keyed.value)
        }
    }

    @ViewBuilder
    private func header(_ group: GemGroup) -> some View {
        if let title = group.title?.text {
            Text(title)
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
        case let .validator(row, isSelected):
            ValidatorSelectionView(row: row, isSelected: isSelected) {
                onSelect(item)
            }
            .ifLet(row.explorerContext) { view, explorerContext in
                view.explorerContext(explorerContext)
            }
        case let .connection(connection):
            ConnectionView(connection: connection)
        case let .priceAlert(alert):
            ListAssetItemView(row: alert.row.row)
        }
    }
}

extension GemListItem {
    var identity: String? {
        switch self {
        case let .currency(row): row.currency.toPrimitives().rawValue
        case let .validator(row, _): row.validator.id
        case let .connection(connection): connection.connection.session.id
        case let .priceAlert(alert): alert.id
        }
    }
}
