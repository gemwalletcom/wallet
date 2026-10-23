// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct ListSectionView<Item: Identifiable & Sendable, Content: View, TrailingRows: View>: View {
    private let sections: [ListSection<Item>]
    private let content: (Item) -> Content
    private let trailingRows: () -> TrailingRows

    public init(
        sections: [ListSection<Item>],
        @ViewBuilder content: @escaping (Item) -> Content,
        @ViewBuilder trailingRows: @escaping () -> TrailingRows,
    ) {
        self.sections = sections
        self.content = content
        self.trailingRows = trailingRows
    }

    public var body: some View {
        List {
            ForEach(sections) { section in
                let sectionContent = {
                    ForEach(section.values) { item in
                        content(item)
                    }
                }

                Section {
                    sectionContent()
                } header: {
                    if section.title == nil, section.image == nil {
                        EmptyView()
                    } else {
                        HStack {
                            if let image = section.image {
                                image
                            }
                            if let title = section.title {
                                Text(title)
                            }
                        }
                    }
                } footer: {
                    if let footer = section.footer {
                        Text(footer)
                    }
                }
            }
            trailingRows()
        }
    }
}

public extension ListSectionView where TrailingRows == EmptyView {
    init(
        sections: [ListSection<Item>],
        @ViewBuilder content: @escaping (Item) -> Content,
    ) {
        self.init(sections: sections, content: content, trailingRows: { EmptyView() })
    }
}
