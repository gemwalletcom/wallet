// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

@Observable
@MainActor
final class CoinPriceRowViewModel {
    private let coin: CoinPrice
    init(coin: CoinPrice) {
        self.coin = coin
    }

    var name: String {
        coin.name
    }

    var symbol: String {
        coin.symbol
    }

    var assetImage: AssetImage {
        AssetImage(
            type: .text(coin.symbol),
            placeholder: coin.image,
            chainPlaceholder: chainPlaceholder,
        )
    }

    var chainPlaceholder: Image? {
        switch coin.assetId.type {
        case .native: nil
        case .token: Images.name(coin.assetId.chain.rawValue)
        }
    }

    var priceText: String {
        coin.priceText
    }

    var percentageText: String {
        coin.changeText
    }

    var percentageColor: Color {
        coin.changeTone.color
    }
}
