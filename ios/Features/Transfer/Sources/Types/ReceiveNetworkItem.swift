// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

struct ReceiveNetworkItem: Hashable, Identifiable, Sendable {
    let assetId: AssetId

    var id: String {
        assetId.identifier
    }

    var chain: Chain {
        assetId.chain
    }
}
