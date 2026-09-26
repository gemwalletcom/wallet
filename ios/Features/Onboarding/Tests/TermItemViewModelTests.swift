// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Onboarding
import Testing

struct TermItemViewModelTests {
    @Test
    func tickingATermChangesHowItReads() {
        let item = TermItemViewModel(message: "I understand")

        let unconfirmed = item.style.color
        item.isConfirmed = true

        #expect(item.id == "I understand")
        #expect(item.style.color != unconfirmed)
    }
}
