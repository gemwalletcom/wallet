// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style

struct StreamStatusItemViewModel {
    let isConnected: Bool

    var title: String { "Stream" }

    var status: String {
        isConnected ? Emoji.greenCircle : Emoji.redCircle
    }

    var listItem: ListItemModel {
        ListItemModel(title: title, subtitle: status)
    }
}
