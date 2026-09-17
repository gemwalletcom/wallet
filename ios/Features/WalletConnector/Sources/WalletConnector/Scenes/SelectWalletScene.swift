// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct SelectWalletScene: View {
    @Environment(\.dismiss) private var dismiss

    @Binding private var model: SelectWalletViewModel

    init(model: Binding<SelectWalletViewModel>) {
        _model = model
    }

    var body: some View {
        SelectableListView(
            model: $model,
            onFinishSelection: { rows in
                model.selectedItems = rows.asSet()
                dismiss()
            },
            listContent: {
                SimpleListItemView(model: $0)
            },
        )
        .navigationTitle(model.title)
    }
}

// MARK: - Actions
