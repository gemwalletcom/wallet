// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Wallet {
    /// Deterministic v4 keystore id (UUID v5 from the wallet id)
    var keystoreId: String {
        keystoreIdForWallet(walletId: id.id)
    }

    /// Legacy v3 keystore id (persisted in externalId), used only to locate the pre-v4 file during migration/delete.
    var legacyV3Id: String {
        externalId ?? id.id
    }
}
