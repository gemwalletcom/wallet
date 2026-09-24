// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPerpetualMarketRow
import func Gemstone.perpetualMarketRows
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PerpetualItemViewModel: ListAssetItemViewable {
    let row: GemPerpetualMarketRow

    var action: ((ListAssetItemAction) -> Void)?

    static func items(_ perpetuals: [PerpetualData]) -> [(data: PerpetualData, model: PerpetualItemViewModel)] {
        zip(perpetuals, perpetualMarketRows(markets: perpetuals.map { $0.toGem() })).map { data, row in
            (data, PerpetualItemViewModel(row: row))
        }
    }

    var name: String {
        row.title
    }

    var symbol: String? {
        .none
    }

    var assetImage: AssetImage {
        AssetImage(icon: row.icon)
    }

    var subtitleView: ListAssetItemSubtitleView {
        guard let price = row.price.price else { return .none }
        return .price(
            price: TextValue(
                text: price.text(),
                style: TextStyle(font: .footnote, color: Colors.gray),
            ),
            priceChangePercentage24h: TextValue(
                text: row.price.change?.text() ?? .empty,
                style: TextStyle(font: .footnote, color: row.price.change?.tone.color ?? Colors.gray),
            ),
        )
    }

    var rightView: ListAssetItemRightView {
        .balance(
            balance: TextValue(
                text: row.volume24h.text(),
                style: TextStyle(font: .body, color: .primary, fontWeight: .semibold),
            ),
            totalFiat: TextValue(
                text: "",
                style: TextStyle(font: .footnote, color: .secondary),
            ),
        )
    }
}
