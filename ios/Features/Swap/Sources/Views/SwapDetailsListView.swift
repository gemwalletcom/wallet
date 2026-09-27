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
                HStack(spacing: .tiny) {
                    Text(rate)
                        .textStyle(.calloutSecondary)
                    if let priceImpact = details.summaryPriceImpact {
                        HStack(spacing: .zero) {
                            Text("(").textStyle(.calloutSecondary)
                            Text(priceImpact.text()).textStyle(TextStyle(font: .callout, color: priceImpact.tone.color))
                            Text(")").textStyle(.calloutSecondary)
                        }
                    }
                }
            }
        }
    }
}
