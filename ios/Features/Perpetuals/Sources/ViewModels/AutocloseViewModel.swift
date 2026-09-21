// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.autocloseFieldState
import class Gemstone.GemAutocloseEstimator
import struct Gemstone.GemAutocloseField
import struct Gemstone.GemAutocloseFieldState
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct AutocloseViewModel {
    private let state: GemAutocloseFieldState

    public init(
        type: TpslType,
        price: Double?,
        estimator: GemAutocloseEstimator,
    ) {
        state = autocloseFieldState(
            field: GemAutocloseField(
                tpslType: type.toGem(),
                price: price,
                originalPrice: nil,
                formattedPrice: nil,
                validation: .valid,
                orderId: nil,
            ),
            estimator: estimator,
            showsErrors: false,
        )
    }

    public var priceTitle: String {
        Localized.Asset.price
    }

    public var title: String {
        state.tpslType.toPrimitives().autocloseTitle
    }

    public var profitTitle: String {
        state.isProfit ? Localized.Perpetual.AutoClose.expectedProfit : Localized.Perpetual.AutoClose.expectedLoss
    }

    public var expectedPnL: String {
        state.estimate?.text ?? Placeholder.empty
    }

    public var roeColor: Color {
        state.tone.color
    }

    public var percentSuggestions: [PercentageSuggestion] {
        state.suggestions.map { PercentageSuggestion(number: $0) }
    }
}
