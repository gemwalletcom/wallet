// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PrimitivesComponents

public struct SlippageSuggestion: SuggestionViewable {
    public let id: UInt32
    public let title: String
    public let inputValue: String

    public init(bps: UInt32, title: String, inputValue: String) {
        id = bps
        self.title = title
        self.inputValue = inputValue
    }
}
