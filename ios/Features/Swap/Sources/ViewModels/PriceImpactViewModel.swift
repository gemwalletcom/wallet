// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSwapPriceImpactRow
import struct Gemstone.SwapPriceImpact
import func Gemstone.swapPriceImpactRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PriceImpactViewModel {
    let fromAssetPrice: AssetPriceValue
    let swapPriceImpact: SwapPriceImpact?

    private var row: GemSwapPriceImpactRow? {
        swapPriceImpact.map { swapPriceImpactRow(impact: $0, paySymbol: fromAssetPrice.asset.symbol) }
    }

    var showPriceImpactWarning: Bool {
        row?.warning != nil
    }

    var showsInSummary: Bool {
        row?.showsInSummary == true
    }

    var highImpactWarningTitle: String {
        Localized.Swap.PriceImpactWarning.title
    }

    var highImpactWarningDescription: String? {
        row?.warning?.text
    }

    func listItem(infoAction: (() -> Void)?) -> ListItemModel? {
        priceImpactText.map { ListItemModel(title: priceImpactTitle, subtitle: $0, subtitleStyle: priceImpactStyle, infoAction: infoAction) }
    }

    var priceImpactTitle: String {
        Localized.Swap.priceImpact
    }

    var priceImpactText: String? {
        row?.value.text()
    }

    var priceImpactStyle: TextStyle {
        TextStyle(font: .callout, color: row?.value.tone.color ?? Colors.gray)
    }
}
