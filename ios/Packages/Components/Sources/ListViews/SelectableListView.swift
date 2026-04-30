// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct SelectableListView<ViewModel: SelectableListAdoptable, Content: View>: View {
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
        switch model.state {
        case .noData:
            if let title = model.emptyStateTitle {
                StateEmptyView(title: title)
            }
        case .loading:
            LoadingView()
        case let .data(type):
            switch type {
            case let .plain(items):
                ListView(
                    items: items,
                    content: contentView,
                )
            case let .section(sections):
                ListSectionView(
                    sections: sections,
                    content: contentView,
                )
            }
        case let .error(error):
            ListItemErrorView(errorTitle: model.errorTitle, error: error)
        }
    }

    @ViewBuilder
    private func contentView(_ item: ViewModel.Item) -> some View {
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
                onFinishSelection?([item])
            }
        }
    }

    private func onSelect(item: ViewModel.Item) {
        model.toggle(item: item)

        switch model.selectionType {
        case .multiSelection:
            break
        case .navigationLink, .checkmark:
            onFinishSelection?(Array(model.selectedItems))
        }
    }
}
