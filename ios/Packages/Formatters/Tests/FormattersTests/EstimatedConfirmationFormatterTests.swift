// Copyright (c). Gem Wallet. All rights reserved.

@testable import Formatters
import Foundation
import Testing

struct EstimatedConfirmationFormatterTests {
    private let formatter = EstimatedConfirmationFormatter(locale: Locale(identifier: "en_US"))

    @Test
    func minutesReadWithTheLocaleUnit() {
        #expect(formatter.string(minutes: 12) == "≈ 12 min")
        #expect(formatter.string(minutes: 1) == "≈ 1 min")
    }
}
