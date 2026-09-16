// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemCandleTooltip
import struct Gemstone.GemCandleTooltipCell
import func Gemstone.candleTooltip
import Components
import Formatters
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct CandleTooltipViewModel {
    private static let titleStyle = TextStyle(font: .caption2, color: Colors.secondaryText, fontWeight: .medium)

    private let tooltip: GemCandleTooltip

    public init(candle: ChartCandleStick) {
        tooltip = candleTooltip(candle: candle.toGem())
    }

    var priceCells: [GemCandleTooltipCell] {
        tooltip.prices
    }

    var summaryCells: [GemCandleTooltipCell] {
        tooltip.summary
    }

    func field(for cell: GemCandleTooltipCell) -> ListItemField {
        ListItemField(
            title: TextValue(text: cell.row.title, style: Self.titleStyle, lineLimit: 1),
            value: TextValue(
                text: cell.value.text(),
                style: TextStyle(font: .caption2.monospacedDigit(), color: cell.value.tone.color, fontWeight: .semibold),
                lineLimit: 1,
            ),
        )
    }
}
