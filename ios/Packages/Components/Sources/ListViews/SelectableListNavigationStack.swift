// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public protocol SelectableListNavigationAdoptable {
    var title: String { get }
}

public struct SelectableListNavigationStack<ViewModel: SelectableListAdoptable & SelectableListNavigationAdoptable, Content: View, TrailingRows: View>: View {
    public typealias ListContent = (ViewModel.Item) -> Content
    public typealias FinishSelection = ([ViewModel.Item]) -> Void

    @Environment(\.dismiss) private var dismiss

    @State private var ownModel: ViewModel
    private let liveModel: Binding<ViewModel>?

    private let onFinishSelection: FinishSelection?
    private let listContent: ListContent
    private let trailingRows: () -> TrailingRows

    public init(
        model: Binding<ViewModel>,
        onFinishSelection: FinishSelection?,
        listContent: @escaping ListContent,
        @ViewBuilder trailingRows: @escaping () -> TrailingRows,
    ) {
        _ownModel = State(initialValue: model.wrappedValue)
        liveModel = model
        self.onFinishSelection = onFinishSelection
        self.listContent = listContent
        self.trailingRows = trailingRows
    }

    private var model: Binding<ViewModel> {
        liveModel ?? $ownModel
    }

    public var body: some View {
        NavigationStack {
            SelectableListView(
                model: model,
                onFinishSelection: onFinishSelection,
                listContent: listContent,
                trailingRows: trailingRows,
            )
            .padding(.bottom, .scene.bottom)
            .contentMargins([.top], .medium, for: .scrollContent)
            .navigationTitle(model.wrappedValue.title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark, action: { dismiss() })
                }
            }
        }
        .sheetPresentation(.forCurrentDeviceSize(expandable: true))
    }
}

public extension SelectableListNavigationStack where TrailingRows == EmptyView {
    init(
        model: ViewModel,
        onFinishSelection: FinishSelection?,
        listContent: @escaping ListContent,
    ) {
        _ownModel = State(initialValue: model)
        liveModel = nil
        self.onFinishSelection = onFinishSelection
        self.listContent = listContent
        trailingRows = { EmptyView() }
    }
}
