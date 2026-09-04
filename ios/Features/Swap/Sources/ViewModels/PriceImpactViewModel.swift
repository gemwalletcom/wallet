// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Localization
import Primitives
import Style
import SwiftUI

struct PriceImpactViewModel {
    let fromAssetPrice: AssetPriceValue
    let swapPriceImpact: Primitives.SwapPriceImpact?

    private let percentFormatter = PercentFormatter.signed

    var showPriceImpactWarning: Bool {
        swapPriceImpact?.isHigh == true
    }

    var highImpactWarningTitle: String {
        Localized.Swap.PriceImpactWarning.title
    }

    var highImpactWarningDescription: String? {
        guard let priceImpactText else { return nil }
        return Localized.Swap.PriceImpactWarning.description(priceImpactText, fromAssetPrice.asset.symbol)
    }

    var priceImpactTitle: String {
        Localized.Swap.priceImpact
    }

    var value: PriceImpactValue? {
        guard let swapPriceImpact else { return nil }

        return PriceImpactValue(
            type: swapPriceImpact.impactType,
            value: percentFormatter.string(swapPriceImpact.percentage),
        )
    }

    var priceImpactText: String? {
        swapPriceImpact.map { PercentFormatter.unsigned.string(abs($0.percentage)) }
    }

    var priceImpactStyle: TextStyle {
        let color = switch value?.type {
        case .low, nil: Colors.secondaryText
        case .medium: Colors.orange
        case .high: Colors.red
        case .positive: Colors.green
        }

        return TextStyle(
            font: .callout,
            color: color,
        )
    }
}
