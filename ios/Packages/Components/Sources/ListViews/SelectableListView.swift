// Copyright (c). Gem Wallet. All rights reserved.

import Style
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
        switch model.filteredState {
        case .noData:
            if let title = model.emptyStateTitle {
                StateEmptyView(title: title)
            }
        case .loading:
            LoadingView()
        case let .data(type):
            if let search = model.search {
                listView(type)
                    .searchable(
                        text: Bindable(search).query,
                        placement: .navigationBarDrawer(displayMode: .always),
                    )
                    .autocorrectionDisabled(true)
                    .scrollDismissesKeyboard(.interactively)
                    .overlay {
                        if type.items.isEmpty {
                            ContentUnavailableView {
                                EmptyContentView(model: search.emptyContent)
                            }
                            .background(UIColor.systemGroupedBackground.color)
                        }
                    }
            } else {
                listView(type)
            }
        case let .error(error):
            ListItemErrorView(errorTitle: model.errorTitle, error: error)
        }
    }

    @ViewBuilder
    private func listView(_ type: SelectableListType<ViewModel.Item>) -> some View {
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
                onSelect(item: item)
            }
        }
    }

    private func onSelect(item: ViewModel.Item) {
        if let selected = model.select(item: item) {
            onFinishSelection?(selected)
        }
    }
}
