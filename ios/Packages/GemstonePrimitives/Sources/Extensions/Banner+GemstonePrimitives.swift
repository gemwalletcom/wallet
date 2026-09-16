// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemBannerContext
import struct Gemstone.GemBannerKey
import Primitives

public extension Primitives.Banner {
    var gemKey: GemBannerKey {
        GemBannerKey(
            walletId: walletId?.id,
            assetId: asset?.id.identifier,
            event: event.toGem(),
        )
    }
}
