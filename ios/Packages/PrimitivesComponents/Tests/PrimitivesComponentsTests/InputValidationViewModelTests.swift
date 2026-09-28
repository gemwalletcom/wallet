// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
@testable import PrimitivesComponents
import Testing

@MainActor
struct InputValidationViewModelTests {
    @Test
    func editClearsError() {
        let model = InputValidationViewModel()
        model.text = "wrong"
        model.update(error: AnyError("error"))

        model.text = "fixed"

        #expect(model.error == nil)
    }

    @Test
    func sameTextKeepsError() {
        let model = InputValidationViewModel()
        model.text = "wrong"
        model.update(error: AnyError("error"))

        model.text = "wrong"

        #expect(model.error != nil)
    }
}
