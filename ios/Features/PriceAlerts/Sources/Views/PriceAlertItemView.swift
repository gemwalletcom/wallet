// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PriceAlertItemView: View {
    let item: PriceAlertItem
    let onDelete: (PriceAlert) -> Void

    var body: some View {
        ListAssetItemView(row: item.row)
            .swipeActions(edge: .trailing) {
                Button(Localized.Common.delete, role: .destructive) {
                    onDelete(item.data.priceAlert)
                }
                .tint(Colors.red)
            }
    }
}
