// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemPriceAlertItem
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PriceAlertItemView: View {
    let item: GemPriceAlertItem
    let onDelete: (PriceAlert) -> Void

    var body: some View {
        ListAssetItemView(row: item.row.row)
            .swipeActions(edge: .trailing) {
                Button(Localized.Common.delete, role: .destructive) {
                    onDelete(item.data.priceAlert.toPrimitives())
                }
                .tint(Colors.red)
            }
    }
}
