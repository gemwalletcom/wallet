// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents
import SwiftUI

struct ChainSelectorView: View {
    @Environment(\.dismiss) private var dismiss

    let model: NetworkSelectorViewModel
    let onSelectChain: (Chain) -> Void

    var body: some View {
        ChainSelectorScene(
            model: model,
            onSelectChain: {
                onSelectChain($0)
                dismiss()
            },
        )
    }
}
