// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemChainServiceProtocol
import Primitives
import SwiftUI

struct ChainSelectorView: View {
    @Environment(\.dismiss) private var dismiss

    let chain: Chain?
    let chainService: any GemChainServiceProtocol
    let onSelectChain: (Chain) -> Void

    var body: some View {
        ChainSelectorScene(
            chain: chain,
            chainService: chainService,
            onSelectChain: {
                onSelectChain($0)
                dismiss()
            },
        )
    }
}
