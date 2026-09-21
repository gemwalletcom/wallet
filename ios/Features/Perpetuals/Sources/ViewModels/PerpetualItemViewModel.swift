// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PerpetualItemViewModel: ListAssetItemViewable {
    let model: PerpetualViewModel

    init(
        model: PerpetualViewModel,
    ) {
        self.model = model
    }

    var name: String {
        model.name
    }

    var symbol: String? {
        .none
    }

    var action: ((ListAssetItemAction) -> Void)?

    var assetImage: AssetImage {
        model.assetImage
    }

    var subtitleView: ListAssetItemSubtitleView {
        guard let price = model.row.price.price else { return .none }
        return .price(
            price: TextValue(
                text: price.text(),
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
                text: model.row.volume24h.text(),
                style: TextStyle(font: .body, color: .primary, fontWeight: .semibold),
            ),
            totalFiat: TextValue(
                text: "",
                style: TextStyle(font: .footnote, color: .secondary),
            ),
        )
    }
}
