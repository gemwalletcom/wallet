// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension AssetId {
    static func mock(
        chain: Chain = .bitcoin,
        tokenId: String? = nil,
    ) -> AssetId {
        AssetId(chain: chain, tokenId: tokenId)
    }
}
