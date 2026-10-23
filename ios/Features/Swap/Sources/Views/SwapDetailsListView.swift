// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemSwapDetails
import Localization
import PrimitivesComponents
import Style
import SwiftUI

public struct SwapDetailsListView: View {
    private let details: GemSwapDetails?
    private let progress: String?

    public init(details: GemSwapDetails?, progress: String? = nil) {
        self.details = details
        self.progress = progress
    }

    public var body: some View {
        HStack {
            ListItemView(model: ListItemModel(title: Localized.Common.details))

            Spacer(minLength: .extraSmall)

            HStack(spacing: .small) {
                if let progress {
                    HStack(spacing: .tiny) {
                        LoadingView(size: .small)
                        Text(progress)
                            .textStyle(TextStyle(font: .footnote, color: Colors.secondaryText, fontWeight: .medium))
                    }
                    .transition(.opacity)
                }
                if let details, let rate = details.rateText(isInverse: false) {
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
            .animation(.easeInOut(duration: .AnimationDuration.normal), value: progress == nil)
        }
    }
}
