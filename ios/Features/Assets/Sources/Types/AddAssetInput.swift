// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAssetConfigService
import Primitives

private let assetConfig = GemAssetConfigService()

struct AddAssetInput {
    let chains: [Chain]

    var chain: Chain?
    var address: String?

    var hasManyChains: Bool {
        chains.count > 1
    }

    init(chains: [Chain]) {
        self.chains = chains
        chain = assetConfig.defaultTokenChain(chains: chains.map(\.rawValue)).flatMap { Chain(rawValue: $0) }
    }
}
