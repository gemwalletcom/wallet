// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Primitives.Wallet {
    /// Legacy v3 keystore id (persisted in externalId), used only to locate the pre-v4 file during migration/delete.
    var legacyV3Id: String {
        externalId ?? id.id
    }

    var chains: [Chain] {
        let walletChains = accounts.map(\.chain).asSet()
        return walletChains.intersection(AssetConfiguration.allChains).asArray().sortByRank()
    }
}
