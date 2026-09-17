// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct CandleTooltipView: View {
    let model: CandleTooltipViewModel

    var body: some View {
        Grid(alignment: .leading, horizontalSpacing: Spacing.small, verticalSpacing: Spacing.extraSmall) {
            ForEach(model.priceCells, id: \.row) {
                GridItemView(field: model.field(for: $0))
            }

            Divider()
                .gridCellColumns(2)
                .padding(.vertical, Spacing.tiny)

            ForEach(model.summaryCells, id: \.row) {
                GridItemView(field: model.field(for: $0))
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
}
