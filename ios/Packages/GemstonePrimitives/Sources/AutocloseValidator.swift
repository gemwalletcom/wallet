// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public struct AutocloseValidator: Sendable {
    private let validator: Gemstone.AutocloseValidator

    public init(type: Primitives.TpslType, direction: Primitives.PerpetualDirection, marketPrice: Double) {
        validator = Gemstone.AutocloseValidator(triggerType: type.map(), direction: direction.map(), marketPrice: marketPrice)
    }

    public func validate(_ price: Double) -> AutocloseValidation {
        validator.validate(price: price)
    }
}
