// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

struct ReceiveNetworkItem: Hashable, Identifiable {
    let assetId: AssetId

    var id: String {
        assetId.identifier
    }
}
