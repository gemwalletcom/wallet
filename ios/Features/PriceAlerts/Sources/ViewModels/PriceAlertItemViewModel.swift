// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

struct PriceAlertItemViewModel: ListAssetItemViewable {
    let data: PriceAlertData
    private let priceModel: PriceViewModel

    init(data: PriceAlertData, currency: String) {
        self.data = data
        let currencyCode = switch data.priceAlert.type {
        case .auto: currency
        case .price, .pricePercentChange: data.priceAlert.currency.rawValue
        }
        priceModel = PriceViewModel(price: data.price, currencyCode: currencyCode)
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
        switch data.priceAlert.type {
        case .auto: priceModel.priceAmountText
        case .price: priceDirectionPrefix
        case .pricePercentChange: percentDirectionPrefix
        }
    }

    private var suffixText: String {
        switch data.priceAlert.type {
        case .auto: priceModel.priceChangeText
        case .price: priceModel.fiatAmountText(amount: data.priceAlert.price ?? .zero)
        case .pricePercentChange: PercentFormatter.unsigned.string(data.priceAlert.pricePercentChange ?? .zero)
        }
    }

    private var priceDirectionPrefix: String {
        switch data.priceAlert.priceDirection {
        case .up: Localized.PriceAlerts.Direction.over
        case .down: Localized.PriceAlerts.Direction.under
        case .none: .empty
        }
    }

    private var percentDirectionPrefix: String {
        switch data.priceAlert.priceDirection {
        case .up: Localized.PriceAlerts.Direction.increasesBy
        case .down: Localized.PriceAlerts.Direction.decreasesBy
        case .none: .empty
        }
    }

    private var directionColor: Color {
        switch data.priceAlert.priceDirection {
        case .up: Colors.green
        case .down: Colors.red
        case .none: priceModel.priceChangeTextColor
        }
    }
}
