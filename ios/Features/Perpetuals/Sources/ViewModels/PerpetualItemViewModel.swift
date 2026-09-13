// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPerpetualMarketRow
import func Gemstone.perpetualMarketRow
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PerpetualItemViewModel: ListAssetItemViewable {
    let model: PerpetualViewModel

    private let row: GemPerpetualMarketRow

    init(
        model: PerpetualViewModel,
    ) {
        self.model = model
        row = perpetualMarketRow(perpetual: model.perpetual.map())
    }

    var name: String {
        row.title
    }

    var symbol: String? {
        .none
    }

    var action: ((ListAssetItemAction) -> Void)?

    var assetImage: AssetImage {
        model.assetImage
    }

    var subtitleView: ListAssetItemSubtitleView {
        guard row.showsPrice else { return .none }
        return .price(
            price: TextValue(
                text: model.priceText,
                style: TextStyle(font: .footnote, color: Colors.gray),
            ),
            priceChangePercentage24h: TextValue(
                text: model.priceChangeText,
                style: TextStyle(font: .footnote, color: model.priceChangeTextColor),
            ),
        )
    }

    var rightView: ListAssetItemRightView {
        .balance(
            balance: TextValue(
                text: model.volumeField.value.text,
                style: TextStyle(font: .body, color: .primary, fontWeight: .semibold),
            ),
            totalFiat: TextValue(
                text: "",
                style: TextStyle(font: .footnote, color: .secondary),
            ),
        )
    }
}
