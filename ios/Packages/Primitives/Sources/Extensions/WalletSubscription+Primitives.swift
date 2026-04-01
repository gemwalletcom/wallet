// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension WalletSubscription {
    var asWalletSubscriptionChains: WalletSubscriptionChains {
        WalletSubscriptionChains(
            walletId: walletId,
            chains: subscriptions.flatMap(\.chains).sorted(),
        )
    }
}
