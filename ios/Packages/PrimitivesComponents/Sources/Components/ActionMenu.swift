// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

@resultBuilder
enum ActionMenuBuilder {
    static func buildBlock(
        _ components: ActionMenuItemType...,
    ) -> [ActionMenuItemType] {
        components
    }
}

struct ActionMenu<Label: View>: View {
    private let items: [ActionMenuItemType]
    private let label: Label

    init(
        items: [ActionMenuItemType],
        @ViewBuilder label: () -> Label,
    ) {
        self.items = items
        self.label = label()
    }

    init(
        @ActionMenuBuilder items: () -> [ActionMenuItemType],
        @ViewBuilder label: () -> Label,
    ) {
        self.init(items: items(), label: label)
    }

    var body: some View {
        Menu {
            ForEach(Array(items.runs(by: \.isDestructive).enumerated()), id: \.offset) { _, group in
                Section {
                    ForEach(group) { ActionMenuItemView(item: $0) }
                }
            }
        } label: {
            label
        }
    }
}
