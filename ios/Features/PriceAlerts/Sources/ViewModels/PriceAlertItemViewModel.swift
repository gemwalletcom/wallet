// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemPriceAlertRow
import class Gemstone.PriceAlertFormatter
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

struct PriceAlertItemViewModel: ListAssetItemViewable {
    private let row: GemPriceAlertRow

    init(data: PriceAlertData, currency: Currency) {
        row = PriceAlertFormatter.shared.row(data: data.toGem(), priceCurrency: currency.toGem())
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
        AssetIdViewModel(assetId: AssetId(core: row.assetId)).assetImage
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
