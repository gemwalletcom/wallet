// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Stake
import Testing

struct ValidatorViewModelTests {
    @Test func aprText() {
        let model = ValidatorViewModel(validator: .mock(apr: 2.15))

        #expect(model.aprModel.text == "APR 2.15%")
    }

    @Test func nameFallsBackToTheAddressWhenUnnamed() {
        let validator = DelegationValidator.mock(.solana, id: "8GbwASqdpw4dVcwbWUxbHXMrjyQx2aKkoBR5H1GJF8iD", name: "")

        #expect(ValidatorViewModel(validator: validator).name == "8GbwA...JF8iD")
        #expect(ValidatorViewModel(validator: .mock(name: "Everstake")).name == "Everstake")
    }
}
