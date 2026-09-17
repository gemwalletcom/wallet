// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
        guard let assetId = try? AssetId(id: coin.coin.assetId) else { return nil }
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
        PriceChangeColor.color(for: coin.coin.change.value)
    }
}
