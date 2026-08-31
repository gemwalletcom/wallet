// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct SearchableSelectableListView<ViewModel: SelectableSearchable, Content: View>: View {
    public typealias ListContent = (ViewModel.Item) -> Content
    public typealias FinishSelection = ([ViewModel.Item]) -> Void

    @Binding private var model: ViewModel

    private let onFinishSelection: FinishSelection?
    private let listContent: ListContent

    public init(
        model: Binding<ViewModel>,
        onFinishSelection: FinishSelection? = nil,
        listContent: @escaping ListContent,
    ) {
        _model = model
        self.listContent = listContent
        self.onFinishSelection = onFinishSelection
    }

    public var body: some View {
        if let search = model.search {
            SearchableListView(
                items: model.items,
                filter: search.filter,
                content: { item in
                    switch model.selectionType {
                    case .multiSelection, .checkmark:
                        SelectionView(
                            value: item,
                            selection: model.selectedItems.contains(item) ? item : nil,
                            action: onSelect(item:),
                            content: {
                                listContent(item)
                            },
                        )
                    case .navigationLink:
                        NavigationCustomLink(with: listContent(item)) {
                            onSelect(item: item)
                        }
                    }
                },
                emptyContent: {
                    EmptyContentView(model: search.emptyContent)
                },
            )
        } else {
            SelectableListView(
                model: $model,
                onFinishSelection: onFinishSelection,
                listContent: listContent,
            )
        }
    }

    private func onSelect(item: ViewModel.Item) {
        if let selected = model.select(item: item) {
            onFinishSelection?(selected)
        }
    }
}
