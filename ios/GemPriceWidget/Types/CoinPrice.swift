// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
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

    var assetImage: AssetImage {
        AssetImage(
            type: .text(symbol),
            placeholder: image,
            chainPlaceholder: chainPlaceholder,
        )
    }

    private var chainPlaceholder: Image? {
        switch assetId.type {
        case .native: nil
        case .token: Images.name(assetId.chain.rawValue)
        }
    }
}
