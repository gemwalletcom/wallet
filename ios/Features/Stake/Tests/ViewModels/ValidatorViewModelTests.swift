// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Stake
import Testing

struct ValidatorViewModelTests {
    @Test func aprText() {
        #expect(mock(.mock(apr: 2.15)).aprModel.text == "APR 2.15%")
    }

    private func mock(_ validator: DelegationValidator) -> ValidatorViewModel {
        ValidatorViewModel(
            row: GemValidatorRow(
                validator: validator.map(),
                name: "",
                imageUrl: "https://assets.gemwallet.com/validator.png",
                placeholder: "",
                provider: .none,
            ),
        )
    }
}
