// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemRewardsRedemption
import PrimitivesComponents

extension GemRewardsRedemption {
    var listItem: ListItemModel {
        ListItemModel(title: title.text, subtitle: points.text(), imageStyle: .asset(assetImage: AssetImage(icon: icon)))
    }
}
