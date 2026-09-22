// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.formattedPercentage
import enum Gemstone.GemPercentageStyle
import GemstonePrimitives
import Localization
import Primitives
import Style

public struct AllTimeValueViewModel: Sendable {
    private let priceFormatter: CurrencyFormatter
    private let percentageStyle: GemPercentageStyle

    public init(priceFormatter: CurrencyFormatter, percentageStyle: GemPercentageStyle) {
        self.priceFormatter = priceFormatter
        self.percentageStyle = percentageStyle
    }

    public func allTimeHigh(chartValue: ChartValuePercentage) -> ListItemModel {
        model(title: Localized.Asset.allTimeHigh, chartValue: chartValue)
    }

    public func allTimeLow(chartValue: ChartValuePercentage) -> ListItemModel {
        model(title: Localized.Asset.allTimeLow, chartValue: chartValue)
    }

    public func model(title: String, chartValue: ChartValuePercentage) -> ListItemModel {
        let percentage = Double(chartValue.percentage)
        return ListItemModel(
            title: title,
            titleExtra: TransactionDateFormatter(date: chartValue.date).section,
            subtitle: priceFormatter.string(Double(chartValue.value)),
            subtitleExtra: Gemstone.formattedPercentage(value: percentage, style: percentageStyle).text(),
            subtitleStyleExtra: TextStyle(font: .callout, color: PriceViewModel.priceChangeTextColor(value: percentage)),
        )
    }
}
