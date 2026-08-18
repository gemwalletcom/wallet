// Copyright (c). Gem Wallet. All rights reserved.

@testable import Formatters
import Foundation
import Testing

struct EstimatedConfirmationFormatterTests {
    private let formatter = EstimatedConfirmationFormatter(locale: Locale(identifier: "en_US"))

    @Test
    func roundedMinutes() {
        #expect(formatter.string(seconds: 720) == "≈ 12 min")
        #expect(formatter.string(seconds: 750) == "≈ 13 min")
    }

    @Test
    func minimumOneMinute() {
        #expect(formatter.string(seconds: 1) == "≈ 1 min")
    }
}
