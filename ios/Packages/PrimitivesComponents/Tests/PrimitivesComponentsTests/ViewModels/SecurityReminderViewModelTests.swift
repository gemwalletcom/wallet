// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import PrimitivesComponents
import Testing

struct SecurityReminderViewModelTests {
    @Test
    func theRemindersComeFromCore() {
        let model = SecurityReminderViewModel(title: "Before you start", onNext: {})

        #expect(model.title == "Before you start")
        #expect(model.items.isNotEmpty)
        #expect(model.message.isNotEmpty)
    }
}
