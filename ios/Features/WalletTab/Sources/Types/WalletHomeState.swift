// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents

struct WalletHomeState {
    let sections: AssetsSections
    let header: WalletHeaderViewModel
    let currencyCode: String
    let showPerpetuals: Bool
    let showCollections: Bool
    let visibleBanners: [Banner]
}
