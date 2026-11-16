// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemWalletRow
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct SelectWalletScene: View {
    @Environment(\.dismiss) private var dismiss

    @Binding private var model: SelectableListViewModel<GemWalletRow>

    init(model: Binding<SelectableListViewModel<GemWalletRow>>) {
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
                ListItemView(model: $0.nameListItem)
            },
        )
        .navigationTitle(model.title)
    }
}

// MARK: - Actions
