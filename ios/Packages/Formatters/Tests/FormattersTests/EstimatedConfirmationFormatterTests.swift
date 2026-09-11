// Copyright (c). Gem Wallet. All rights reserved.

@testable import Formatters
import Foundation
import Testing

struct EstimatedConfirmationFormatterTests {
    private let formatter = EstimatedConfirmationFormatter(locale: Locale(identifier: "en_US"))

    @Test
    func minutesKeepTheSecondsUnderThem() {
        #expect(formatter.string(seconds: 720) == "≈ 12 min")
        #expect(formatter.string(seconds: 90) == "≈ 1 min, 30 sec")
        #expect(formatter.string(seconds: 45) == "≈ 45 sec")
    }
}
