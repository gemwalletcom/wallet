// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSwapDetails
import struct Gemstone.GemSwapQuoteProgress
import Localization
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapDetailsListView: View {
    private let details: GemSwapDetails
    private let progress: GemSwapQuoteProgress?

    public init(details: GemSwapDetails, progress: GemSwapQuoteProgress? = nil) {
        self.details = details
        self.progress = progress
    }

    public var body: some View {
        HStack {
            ListItemView(model: ListItemModel(title: Localized.Common.details))

            Spacer(minLength: .extraSmall)

            HStack(spacing: .tiny) {
                if let progress {
                    SwapQuoteProgressView(progress: progress)
                }
                if let rate = details.rateText(isInverse: false) {
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
