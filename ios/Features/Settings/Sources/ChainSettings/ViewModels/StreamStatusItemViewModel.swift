// Copyright (c). Gem Wallet. All rights reserved.

import Style

struct StreamStatusItemViewModel {
    let isConnected: Bool

    var title: String { "Stream" }

    var status: String {
        isConnected ? Emoji.greenCircle : Emoji.redCircle
    }
}
