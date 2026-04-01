// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

struct ChainSelectorScene: View {
    @State private var model: NetworkSelectorViewModel

    private let onSelectChain: (Chain) -> Void

    init(chain: Chain?, onSelectChain: @escaping (Chain) -> Void) {
        _model = State(initialValue: NetworkSelectorViewModel(
            state: .data(.plain(Chain.allCases)),
            selectedItems: [chain].compactMap(\.self),
            selectionType: .checkmark,
        ))
        self.onSelectChain = onSelectChain
    }

    var body: some View {
        SearchableSelectableListView(
            model: $model,
            onFinishSelection: { chains in
                if let chain = chains.first {
                    onSelectChain(chain)
                }
            },
            listContent: { ChainView(model: ChainViewModel(chain: $0)) },
        )
        .navigationTitle(Localized.Transfer.network)
    }
}
