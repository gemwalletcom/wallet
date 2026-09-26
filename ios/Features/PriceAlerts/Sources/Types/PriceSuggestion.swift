// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PrimitivesComponents

struct PriceSuggestion: SuggestionViewable {
    let title: String
    let inputValue: String

    var id: String {
        "\(title)_\(inputValue)"
    }
}
