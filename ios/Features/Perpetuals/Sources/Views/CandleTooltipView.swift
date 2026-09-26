// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemCandleTooltip
import struct Gemstone.GemCandleTooltipCell
import PrimitivesComponents
import Style
import SwiftUI

struct CandleTooltipView: View {
    private static let titleStyle = TextStyle(font: .caption2, color: Colors.secondaryText, fontWeight: .medium)

    let tooltip: GemCandleTooltip

    var body: some View {
        Grid(alignment: .leading, horizontalSpacing: Spacing.small, verticalSpacing: Spacing.extraSmall) {
            ForEach(tooltip.prices, id: \.row) {
                GridItemView(field: field(for: $0))
            }

            Divider()
                .gridCellColumns(2)
                .padding(.vertical, Spacing.tiny)

            ForEach(tooltip.summary, id: \.row) {
                GridItemView(field: field(for: $0))
            }
        }
        .padding(Spacing.small)
        .background(.thickMaterial)
        .clipShape(RoundedRectangle(cornerRadius: Spacing.small))
        .overlay(
            RoundedRectangle(cornerRadius: Spacing.small)
                .stroke(Colors.black.opacity(.opacity8), lineWidth: .space1),
        )
        .shadow(color: .black.opacity(.opacity12), radius: Spacing.small, y: Spacing.tiny)
        .fixedSize()
    }

    private func field(for cell: GemCandleTooltipCell) -> ListItemField {
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
