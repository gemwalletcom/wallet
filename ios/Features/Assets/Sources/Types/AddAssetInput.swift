// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

struct AddAssetInput {
    let chains: [Chain]
    let showsChainPicker: Bool

    var chain: Chain?
    var address: String?

    init(chains: [Chain], chain: Chain?, showsChainPicker: Bool) {
        self.chains = chains
        self.chain = chain
        self.showsChainPicker = showsChainPicker
    }
}
