// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct NetworkSelectorScene: View {
    @Environment(\.dismiss) private var dismiss

    @State private var model: NetworkSelectorViewModel

    private let onFinishSelection: (([Chain]) -> Void)?

    public init(
        model: NetworkSelectorViewModel,
        onFinishSelection: (([Chain]) -> Void)? = nil,
    ) {
        _model = State(initialValue: model)
        self.onFinishSelection = onFinishSelection
    }

    public var body: some View {
        SelectableListView(
            model: $model,
            onFinishSelection: { chains in
                onFinishSelection?(chains)
                dismiss()
            },
            listContent: { ChainView(model: ChainViewModel(chain: $0)) },
        )
        .navigationTitle(model.title)
    }
}
