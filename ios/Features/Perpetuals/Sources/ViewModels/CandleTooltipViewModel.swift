// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemCandleTooltip
import struct Gemstone.GemFormattedNumber
import func Gemstone.candleTooltip
import Components
import Formatters
import Localization
import Primitives
import Style
import SwiftUI

public struct CandleTooltipViewModel {
    private static let titleStyle = TextStyle(font: .caption2, color: Colors.secondaryText, fontWeight: .medium)
    private static let subtitleStyle = TextStyle(font: .caption2.monospacedDigit(), color: Colors.black, fontWeight: .semibold)

    private let tooltip: GemCandleTooltip

    public init(candle: ChartCandleStick) {
        tooltip = candleTooltip(candle: candle.toGem())
    }

    var openField: ListItemField {
        field(title: Localized.Charts.Price.open, value: tooltip.open)
    }

    var closeField: ListItemField {
        field(title: Localized.Charts.Price.close, value: tooltip.close)
    }

    var highField: ListItemField {
        field(title: Localized.Charts.Price.high, value: tooltip.high)
    }

    var lowField: ListItemField {
        field(title: Localized.Charts.Price.low, value: tooltip.low)
    }

    var changeField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Charts.Price.change, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(
                text: tooltip.change.text(),
                style: TextStyle(font: .caption2.monospacedDigit(), color: PriceChangeColor.color(for: tooltip.change.value), fontWeight: .semibold),
                lineLimit: 1,
            ),
        )
    }

    var volumeField: ListItemField {
        field(title: Localized.Perpetual.volume, value: tooltip.volume)
    }

    private func field(title: String, value: GemFormattedNumber) -> ListItemField {
        ListItemField(
            title: TextValue(text: title, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(text: value.text(), style: Self.subtitleStyle, lineLimit: 1),
        )
    }
}
