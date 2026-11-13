// Copyright (c). Gem Wallet. All rights reserved.

import Components

struct LightningReceiveTrigger: DebouncableTrigger {
    let amount: String

    var isImmediate: Bool {
        amount.isEmpty
    }
}
