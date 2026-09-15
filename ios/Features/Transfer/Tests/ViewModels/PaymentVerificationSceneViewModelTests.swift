// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
@testable import Transfer

@MainActor
struct PaymentVerificationSceneViewModelTests {
    @Test
    func completedFormReportsBack() throws {
        var completed = 0
        let model = try PaymentVerificationSceneViewModel.mock { completed += 1 }

        model.onMessage(["type": "IC_COMPLETE"])

        #expect(completed == 1)
    }

    @Test
    func onlyTheCompleteMessageCounts() throws {
        var completed = 0
        let model = try PaymentVerificationSceneViewModel.mock { completed += 1 }

        model.onMessage(["type": "IC_ERROR"])

        #expect(completed == 0)
    }
}

private extension PaymentVerificationSceneViewModel {
    static func mock(onComplete: @escaping () -> Void) throws -> PaymentVerificationSceneViewModel {
        PaymentVerificationSceneViewModel(url: try #require(URL(string: "https://walletconnect.com/collect")), onComplete: onComplete)
    }
}
