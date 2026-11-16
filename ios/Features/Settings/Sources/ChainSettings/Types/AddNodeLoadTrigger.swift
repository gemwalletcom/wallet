// Copyright (c). Gem Wallet. All rights reserved.

import Components

struct AddNodeLoadTrigger: DebouncableTrigger {
    let url: String
    let isImmediate: Bool
}
