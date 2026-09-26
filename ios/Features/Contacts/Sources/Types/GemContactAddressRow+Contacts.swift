// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemContactAddressRow
import PrimitivesComponents

extension GemContactAddressRow {
    var listItem: ListItemModel {
        ListItemModel(
            title: chain.title,
            titleExtra: shortAddress,
            imageStyle: .asset(assetImage: AssetImage(icon: chain.icon)),
        )
    }
}
