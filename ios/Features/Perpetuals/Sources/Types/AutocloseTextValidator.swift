// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AutocloseValidation
import class Gemstone.AutocloseValidator
import GemstonePrimitives
import Primitives
import PrimitivesComponents

struct AutocloseTextValidator: TextValidator {
    private let validator: AutocloseValidator

    init(type: TpslType, direction: PerpetualDirection, marketPrice: Double) {
        validator = AutocloseValidator(triggerType: type.toGem(), direction: direction.toGem(), marketPrice: marketPrice)
    }

    func validate(_ text: String) throws {
        guard !text.isEmpty else { return }

        guard let price = NumberInput.double(text) else {
            throw AutocloseValidation.invalidAmount
        }

        let validation = validator.validate(price: price)
        guard validation == .valid else {
            throw validation
        }
    }

    var id: String {
        "AutocloseTextValidator"
    }
}
