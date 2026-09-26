// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSearchListRow

extension GemSearchListRow {
    var listItem: ListItemModel {
        ListItemModel(
            title: list.name,
            subtitle: subtitle,
            imageStyle: .settings(assetImage: AssetImage(type: .text(list.name), imageURL: URL(string: imageUrl))),
        )
    }
}
