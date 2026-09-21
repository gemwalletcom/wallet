// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.DelegationValidator
import struct Gemstone.GemFormattedNumber
import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension GemValidatorRow {
    static func mock(
        validator: Gemstone.DelegationValidator = Primitives.DelegationValidator.mock().toGem(),
        apr: GemFormattedNumber? = nil,
    ) -> GemValidatorRow {
        GemValidatorRow(
            validator: validator,
            name: validator.name,
            imageUrl: "https://assets.gemwallet.com/validator.png",
            placeholder: String(validator.name.prefix(1)),
            provider: .none,
            apr: apr,
        )
    }
}
