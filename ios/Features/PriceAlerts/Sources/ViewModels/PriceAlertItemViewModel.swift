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
    let data: PriceAlertData
    private let priceModel: PriceViewModel
    private let row: GemPriceAlertRow

    init(data: PriceAlertData, currency: String) {
        self.data = data
        let currencyCode = switch data.priceAlert.type {
        case .auto: currency
        case .price, .pricePercentChange: data.priceAlert.currency.rawValue
        }
        priceModel = PriceViewModel(price: data.price, currencyCode: currencyCode)
        row = PriceAlertFormatter.shared.row(
            alert: data.priceAlert.map(),
            currentPrice: data.price?.price,
            priceChangePercentage24h: data.price?.priceChangePercentage24h,
        )
    }

    var name: String {
        data.asset.name
    }

    var symbol: String? {
        data.asset.symbol
    }

    var rightView: ListAssetItemRightView {
        .none
    }

    var action: ((ListAssetItemAction) -> Void)?

    var assetImage: AssetImage {
        AssetViewModel(asset: data.asset).assetImage
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
            text: prefixText,
            style: TextStyle(font: .footnote, color: Colors.gray),
        )
    }

    private var suffixTextValue: TextValue {
        TextValue(
            text: suffixText,
            style: TextStyle(font: .footnote, color: directionColor),
        )
    }

    private var prefixText: String {
        switch row.kind {
        case .auto: priceModel.priceAmountText
        case .over: Localized.PriceAlerts.Direction.over
        case .under: Localized.PriceAlerts.Direction.under
        case .increase: Localized.PriceAlerts.Direction.increasesBy
        case .decrease: Localized.PriceAlerts.Direction.decreasesBy
        }
    }

    private var suffixText: String {
        switch row.kind {
        case .auto: priceModel.priceChangeText
        case .over, .under: priceModel.fiatAmountText(amount: data.priceAlert.price ?? .zero)
        case .increase, .decrease: PercentFormatter.unsigned.string(data.priceAlert.pricePercentChange ?? .zero)
        }
    }

    private var directionColor: Color {
        switch row.direction {
        case .up: Colors.green
        case .down: Colors.red
        case .none: priceModel.priceChangeTextColor
        }
    }
}
