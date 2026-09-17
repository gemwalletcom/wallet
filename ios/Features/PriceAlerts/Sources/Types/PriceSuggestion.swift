// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import PrimitivesComponents

struct PriceSuggestion: SuggestionViewable {
    let title: String
    let value: Double

    var inputValue: String {
        NumberInput.format().valueText(value: value)
    }

    var id: String {
        "\(title)_\(inputValue)"
    }
}
