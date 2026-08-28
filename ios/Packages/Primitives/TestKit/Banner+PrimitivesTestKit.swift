// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Banner {
    static func mock(
        walletId: WalletId? = .mock(),
        asset: Asset? = .mock(),
        event: BannerEvent = .stake,
        state: BannerState = .active,
    ) -> Banner {
        Banner(
            walletId: walletId,
            asset: asset,
            event: event,
            state: state,
        )
    }
}
