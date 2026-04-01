// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Style
import SwiftUI

public struct SwapDetailsListView: View {
    private let model: SwapDetailsViewModel

    public init(model: SwapDetailsViewModel) {
        self.model = model
    }

    public var body: some View {
        HStack {
            ListItemView(title: Localized.Common.details)

            Spacer(minLength: .extraSmall)

            if let rate = model.rateText {
                HStack(spacing: .tiny) {
                    Text(rate)
                        .textStyle(.calloutSecondary)
                    if model.shouldShowPriceImpactInDetails, let value = model.priceImpactValue {
                        HStack(spacing: .zero) {
                            Text("(").textStyle(.calloutSecondary)
                            Text(value).textStyle(model.priceImpactModel.priceImpactStyle)
                            Text(")").textStyle(.calloutSecondary)
                        }
                    }
                }
            }
        }
    }
}
