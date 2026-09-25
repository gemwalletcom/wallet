// Copyright (c). Gem Wallet. All rights reserved.

import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct PaymentVerificationSceneViewModelTests {
    @Test
    func theFormReportsCompletionAndFailure() {
        var completed = 0
        var failed = 0
        let model = PaymentVerificationSceneViewModel.mock(onComplete: { completed += 1 }, onError: { failed += 1 })

        model.onMessage(["type": "IC_PROGRESS"])
        #expect(completed == 0 && failed == 0)

        model.onMessage(["type": "IC_ERROR"])
        #expect(failed == 1)

        model.onMessage(["type": "IC_COMPLETE"])
        #expect(completed == 1)
    }
}
