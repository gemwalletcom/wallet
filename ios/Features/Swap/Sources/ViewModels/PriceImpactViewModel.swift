// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Gemstone
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

struct PriceImpactViewModel {
    let fromAssetPrice: AssetPriceValue
    let fromValue: String
    let toAssetPrice: AssetPriceValue
    let toValue: String

    private let valueFormatter = ValueFormatter(style: .full)
    private let percentFormatter = CurrencyFormatter.percent

    var showPriceImpactWarning: Bool {
        isHighPriceImpact
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
            type: swapPriceImpact.impactType.map(),
            value: percentFormatter.string(swapPriceImpact.percentage),
        )
    }

    var priceImpactText: String? {
        swapPriceImpact.map { CurrencyFormatter.percentSignLess.string(abs($0.percentage)) }
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

// MARK: - Private

extension PriceImpactViewModel {
    private var isHighPriceImpact: Bool {
        swapPriceImpact?.isHigh == true
    }

    private var swapPriceImpact: Gemstone.SwapPriceImpact? {
        guard
            let fromAmount = getSwapAmount(value: fromValue, decimals: fromAssetPrice.asset.decimals.asInt),
            let toAmount = getSwapAmount(value: toValue, decimals: toAssetPrice.asset.decimals.asInt),
            let fromValue = assetFiatValue(value: fromAmount, price: fromAssetPrice.price?.price),
            let toValue = assetFiatValue(value: toAmount, price: toAssetPrice.price?.price)
        else {
            return nil
        }

        return GemstonePrimitives.calculateSwapPriceImpact(payFiatValue: fromValue, receiveFiatValue: toValue)
    }

    private func getSwapAmount(value: String, decimals: Int) -> Double? {
        guard
            let value = try? BigInt.from(string: value),
            let amount = try? valueFormatter.double(from: value, decimals: decimals)
        else {
            return nil
        }
        return amount
    }

    private func assetFiatValue(value: Double, price: Double?) -> Double? {
        guard let price else {
            return nil
        }
        return value * price
    }
}
