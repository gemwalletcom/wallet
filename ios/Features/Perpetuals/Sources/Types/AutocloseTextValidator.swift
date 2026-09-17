// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
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
            throw PerpetualError.invalidAmount
        }

        switch validator.validate(price: price) {
        case .valid:
            break
        case .invalidAmount:
            throw PerpetualError.invalidAmount
        case .triggerMustBeHigher:
            throw PerpetualError.triggerPriceMustBeHigher
        case .triggerMustBeLower:
            throw PerpetualError.triggerPriceMustBeLower
        }
    }

    var id: String {
        "AutocloseTextValidator"
    }
}
