// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PrimitivesComponents

public struct SlippageSuggestion: SuggestionViewable {
    public let id: UInt32
    private let percentText: String

    public var title: String {
        "\(percentText)%"
    }

    public var inputValue: String {
        percentText
    }

    public init(bps: UInt32, percentText: String) {
        id = bps
        self.percentText = percentText
    }
}
