// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct SearchableSelectableListView<ViewModel: SelectableListAdoptable & ItemFilterable, Content: View>: View {
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
        SearchableListView(
            items: model.items,
            filter: model.filter(_:query:),
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
                if let model = model.emptyContentModel {
                    EmptyContentView(model: model)
                }
            },
        )
    }

    private func onSelect(item: ViewModel.Item) {
        if let selected = model.select(item: item) {
            onFinishSelection?(selected)
        }
    }
}
