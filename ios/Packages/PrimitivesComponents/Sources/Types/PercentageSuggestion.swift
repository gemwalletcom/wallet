// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemFormattedNumber
import Foundation

public struct PercentageSuggestion: SuggestionViewable {
    public let id: Int
    public let value: Int
    public let title: String

    public var inputValue: String {
        String(value)
    }

    public init(value: Int) {
        id = value
        self.value = value
        title = "\(value)%"
    }

    public init(number: GemFormattedNumber) {
        value = Int(number.value)
        id = value
        title = number.text()
    }
}
