// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents

struct WalletHomeState {
    let sections: AssetsSections
    let header: WalletHeaderViewModel
    let currency: Currency
    let showPerpetuals: Bool
    let showCollections: Bool
    let visibleBanners: [Banner]
}
