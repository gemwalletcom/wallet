// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemPriceAlertRow
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

struct PriceAlertItemViewModel: ListAssetItemViewable {
    private let row: GemPriceAlertRow

    init(row: GemPriceAlertRow) {
        self.row = row
    }

    var name: String {
        row.title
    }

    var symbol: String? {
        row.symbol
    }

    var rightView: ListAssetItemRightView {
        .none
    }

    var action: ((ListAssetItemAction) -> Void)?

    var assetImage: AssetImage {
        AssetImage(icon: row.icon)
    }

    var subtitleView: ListAssetItemSubtitleView {
        .price(
            price: prefixTextValue,
            priceChangePercentage24h: suffixTextValue,
        )
    }

    // MARK: - Private

    private var prefixTextValue: TextValue {
        TextValue(
            text: row.prefixText,
            style: TextStyle(font: .footnote, color: Colors.gray),
        )
    }

    private var suffixTextValue: TextValue {
        TextValue(
            text: row.suffixText,
            style: TextStyle(font: .footnote, color: row.directionColor),
        )
    }
}
