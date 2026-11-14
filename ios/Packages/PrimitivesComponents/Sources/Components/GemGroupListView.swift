// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemGroup
import struct Gemstone.GemGroupList
import enum Gemstone.GemListItem
import enum Gemstone.GemListRow
import Style
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
    private let rowInsets: EdgeInsets?
    private let item: (GemListItem) -> Item

    public init(groups: [GemGroup], rowInsets: EdgeInsets? = nil, @ViewBuilder item: @escaping (GemListItem) -> Item) {
        self.groups = groups
        self.rowInsets = rowInsets
        self.item = item
    }

    public var body: some View {
        ForEach(keyedGroups) { keyed in
            let group = keyed.value
            if let footer = group.footer.text {
                Section {
                    items(group)
                } header: {
                    header(group)
                } footer: {
                    Text(footer)
                }
                .ifLet(rowInsets) { $0.listRowInsets($1) }
            } else {
                Section {
                    items(group)
                } header: {
                    header(group)
                }
                .ifLet(rowInsets) { $0.listRowInsets($1) }
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
            if case let .sectionTitle(sectionTitle) = group.title, let image = sectionTitle.image {
                HStack {
                    image
                        .frame(size: .image.small)
                    Text(title)
                }
            } else {
                Text(title)
            }
        }
    }

    private var keyedGroups: [Keyed<GemGroup>] {
        groups.enumerated().map { index, group in
            Keyed(id: group.id ?? "#\(index)", value: group)
        }
    }

    private func keyedItems(_ group: GemGroup) -> [Keyed<GemListItem>] {
        group.items.enumerated().map { index, item in
            Keyed(id: item.identity ?? "#\(index)", value: item)
        }
    }
}

public extension GemGroupSections {
    init<Row: View>(rows groups: [GemGroup], rowInsets: EdgeInsets? = nil, @ViewBuilder row: @escaping (GemListRow) -> Row) where Item == Row? {
        self.init(groups: groups, rowInsets: rowInsets) { item in
            if case let .row(value) = item {
                row(value)
            }
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
        case let .row(row):
            GemListRowView(row: row)
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
        case .connection, .priceAlert, .recipient, .transactionDetail, .node, .explorer:
            EmptyView()
        }
    }
}

extension GemListItem {
    var identity: String? {
        switch self {
        case .row: nil
        case let .currency(row): row.currency.toPrimitives().rawValue
        case let .validator(row, _): row.validator.id
        case let .connection(connection): connection.connection.session.id
        case let .priceAlert(alert): alert.id
        case let .recipient(row): row.recipient.id
        case let .transactionDetail(item): item.id
        case let .node(row): row.node.url
        case let .explorer(row): row.name
        }
    }
}
