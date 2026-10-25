// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSwapDetails
import Localization
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapDetailsListView: View {
    private let details: GemSwapDetails

    public init(details: GemSwapDetails) {
        self.details = details
    }

    public var body: some View {
        HStack {
            ListItemView(model: ListItemModel(title: Localized.Common.details))

            Spacer(minLength: .extraSmall)

            if let rate = details.rateText(isInverse: false) {
                rateText(rate)
                    .textStyle(.calloutSecondary)
                    .lineLimit(1)
                    .minimumScaleFactor(12 / 16)
            }
        }
    }

    private func rateText(_ rate: String) -> Text {
        guard let priceImpact = details.summaryPriceImpact else {
            return Text(rate)
        }
        return Text("\(rate) (\(Text(priceImpact.text()).foregroundStyle(priceImpact.tone.color)))")
    }
}
