// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemBannerRow
import PrimitivesComponents

struct WalletHomeState {
    let sections: AssetsSections
    let header: WalletHeaderViewModel
    let showPerpetuals: Bool
    let showCollections: Bool
    let visibleBanners: [GemBannerRow]
}
