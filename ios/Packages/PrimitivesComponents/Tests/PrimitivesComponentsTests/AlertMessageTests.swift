// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemServiceError
import Primitives
@testable import PrimitivesComponents
import Testing

struct AlertMessageTests {
    @Test
    func failureBecomesAnAlertWithItsDescription() {
        let alert = AlertMessage(title: "Create", error: AnyError("disk full"))

        #expect(alert?.title == "Create")
        #expect(alert?.message == AnyError("disk full").localizedDescription)
    }

    @Test
    func cancellationIsNotAnAlert() {
        #expect(AlertMessage(error: GemServiceError.Cancelled) == nil)
        #expect(AlertMessage(error: CancellationError()) == nil)
    }
}
