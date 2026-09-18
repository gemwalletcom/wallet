// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.valueTone
import enum Gemstone.GemValueTone
import Formatters
import GemstonePrimitives
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
        coin.coin.name
    }

    var symbol: String {
        coin.coin.symbol
    }

    var assetImage: AssetImage {
        AssetImage(
            type: .text(coin.coin.symbol),
            placeholder: coin.image,
            chainPlaceholder: chainPlaceholder,
        )
    }

    var chainPlaceholder: Image? {
        let assetId = AssetId(core: coin.coin.assetId)
        switch assetId.type {
        case .native: return nil
        case .token: return Images.name(assetId.chain.rawValue)
        }
    }

    var priceText: String {
        coin.coin.price.text()
    }

    var percentageText: String {
        coin.coin.change.text()
    }

    var percentageColor: Color {
        valueTone(value: coin.coin.change.value).color
    }
}

private extension GemValueTone {
    var color: Color {
        switch self {
        case .plain: Colors.black
        case .neutral: Colors.gray
        case .positive: Colors.green
        case .warning: Colors.orange
        case .negative: Colors.red
        }
    }
}
