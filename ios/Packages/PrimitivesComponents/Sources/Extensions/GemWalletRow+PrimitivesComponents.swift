// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemWalletRow

public extension GemWalletRow {
    var listItem: ListItemModel {
        ListItemModel(title: name, titleExtra: subtitle.text, imageStyle: .asset(assetImage: avatarImage))
    }

    var nameListItem: ListItemModel {
        ListItemModel(title: name, imageStyle: .asset(assetImage: avatarImage))
    }
}

extension GemWalletRow: @retroactive Identifiable {}
