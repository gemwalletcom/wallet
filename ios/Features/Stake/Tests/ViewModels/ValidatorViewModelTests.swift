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
        #expect(model(.mock(apr: 2.15)).aprModel.text == "APR 2.15%")
    }

    @Test func nameAndPlaceholderComeFromTheRow() {
        let model = model(.mock(name: "Everstake"), name: "Everstake", placeholder: "E")

        #expect(model.name == "Everstake")
        #expect(model.validatorImage.type == .text("E"))
    }

    private func model(_ validator: DelegationValidator, name: String = "", placeholder: String = "") -> ValidatorViewModel {
        ValidatorViewModel(
            row: GemValidatorRow(
                validator: validator.map(),
                name: name,
                imageUrl: "https://assets.gemwallet.com/validator.png",
                placeholder: placeholder,
                provider: .none,
            ),
        )
    }
}
