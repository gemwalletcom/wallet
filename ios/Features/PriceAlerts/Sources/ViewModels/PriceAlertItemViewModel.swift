// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemPriceAlertItem
import GemstonePrimitives
import Primitives

struct PriceAlertItemViewModel: Identifiable {
    let data: PriceAlertData
    let row: GemAssetItemRow

    init(item: GemPriceAlertItem) {
        data = item.data.toPrimitives()
        row = item.row.row
    }

    var id: String {
        data.id
    }
}
