// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style

public struct AllTimeValueViewModel: Sendable {
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter: PercentFormatter
    private let hideBalance: Bool

    public init(priceFormatter: CurrencyFormatter, percentFormatter: PercentFormatter, hideBalance: Bool = false) {
        self.priceFormatter = priceFormatter
        self.percentFormatter = percentFormatter
        self.hideBalance = hideBalance
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
            subtitle: priceFormatter.string(Double(chartValue.value)).masked(if: hideBalance),
            subtitleExtra: percentFormatter.string(percentage),
            subtitleStyleExtra: TextStyle(font: .callout, color: PriceViewModel.priceChangeTextColor(value: percentage)),
        )
    }
}
