// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import class Gemstone.AutocloseValidator
import GemstonePrimitives
import Primitives
import Validators

struct AutocloseTextValidator: TextValidator {
    private let validator: AutocloseValidator
    private let formatter = NumericFormatter()

    init(type: TpslType, direction: PerpetualDirection, marketPrice: Double) {
        validator = AutocloseValidator(triggerType: type.map(), direction: direction.map(), marketPrice: marketPrice)
    }

    func validate(_ text: String) throws {
        guard !text.isEmpty else { return }

        guard let price = formatter.double(from: text) else {
            throw TransferError.invalidAmount
        }

        switch validator.validate(price: price) {
        case .valid:
            break
        case .invalidAmount:
            throw TransferError.invalidAmount
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
