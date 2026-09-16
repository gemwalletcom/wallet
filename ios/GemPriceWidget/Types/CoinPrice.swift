// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemWidgetCoin
import SwiftUI

struct CoinPrice: Identifiable {
    let coin: GemWidgetCoin
    let image: Image?

    var id: String {
        coin.assetId
    }
}
