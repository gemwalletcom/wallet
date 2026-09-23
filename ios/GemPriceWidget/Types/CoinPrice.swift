// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI

struct CoinPrice: Identifiable {
    let assetId: AssetId
    let name: String
    let symbol: String
    let priceText: String
    let changeText: String
    let changeTone: WidgetValueTone
    let image: Image?

    var id: String {
        assetId.identifier
    }
}
