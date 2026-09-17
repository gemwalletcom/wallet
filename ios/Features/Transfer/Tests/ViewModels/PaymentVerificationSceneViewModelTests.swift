// Copyright (c). Gem Wallet. All rights reserved.

import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct PaymentVerificationSceneViewModelTests {
    @Test
    func onlyTheCompleteMessageReportsBack() {
        var completed = 0
        let model = PaymentVerificationSceneViewModel.mock { completed += 1 }

        model.onMessage(["type": "IC_ERROR"])
        #expect(completed == 0)

        model.onMessage(["type": "IC_COMPLETE"])
        #expect(completed == 1)
    }
}
