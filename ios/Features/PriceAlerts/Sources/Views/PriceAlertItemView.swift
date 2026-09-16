// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PriceAlertItemView: View {
    let alert: PriceAlertData
    let currency: Currency
    let onDelete: (PriceAlert) -> Void

    var body: some View {
        ListAssetItemView(model: PriceAlertItemViewModel(data: alert, currency: currency))
            .swipeActions(edge: .trailing) {
                Button(Localized.Common.delete, role: .destructive) {
                    onDelete(alert.priceAlert)
                }
                .tint(Colors.red)
            }
    }
}
