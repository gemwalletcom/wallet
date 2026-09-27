// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemBannerRow
import PrimitivesComponents

struct WalletHomeState {
    let sections: AssetsSections
    let header: ValueHeader
    let showPerpetuals: Bool
    let showCollections: Bool
    let banner: GemBannerRow?
}
