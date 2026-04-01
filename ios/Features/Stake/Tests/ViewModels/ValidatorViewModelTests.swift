// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PrimitivesTestKit
import Stake
import Testing

struct ValidatorViewModelTests {
    @Test func aprText() {
        let model = ValidatorViewModel(validator: .mock(apr: 2.15))

        #expect(model.aprModel.text == "APR 2.15%")
    }
}
