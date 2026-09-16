// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AutocloseValidation
import class Gemstone.AutocloseValidator
import struct Gemstone.GemAutocloseField
import GemstonePrimitives
import Primitives
import PrimitivesComponents

@MainActor
struct AutocloseInput {
    var takeProfit: InputValidationViewModel
    var stopLoss: InputValidationViewModel
    var focusField: AutocloseScene.Field?

    private let takeProfitValidator: AutocloseValidator
    private let stopLossValidator: AutocloseValidator

    init(type: AutocloseType, takeProfitText: String?, stopLossText: String?) {
        takeProfitValidator = AutocloseValidator(triggerType: TpslType.takeProfit.toGem(), direction: type.direction.toGem(), marketPrice: type.marketPrice)
        stopLossValidator = AutocloseValidator(triggerType: TpslType.stopLoss.toGem(), direction: type.direction.toGem(), marketPrice: type.marketPrice)
        takeProfit = InputValidationViewModel(
            mode: .manual,
            validators: [AutocloseTextValidator(type: .takeProfit, direction: type.direction, marketPrice: type.marketPrice)],
        )
        stopLoss = InputValidationViewModel(
            mode: .manual,
            validators: [AutocloseTextValidator(type: .stopLoss, direction: type.direction, marketPrice: type.marketPrice)],
        )

        takeProfitText.map { takeProfit.text = $0 }
        stopLossText.map { stopLoss.text = $0 }
    }

    var selection: AutocloseSelection {
        AutocloseSelection(
            takeProfit: takeProfit.text.isEmpty ? nil : takeProfit.text,
            stopLoss: stopLoss.text.isEmpty ? nil : stopLoss.text,
        )
    }

    var focused: InputValidationViewModel? {
        switch focusField {
        case .takeProfit: takeProfit
        case .stopLoss: stopLoss
        case nil: nil
        }
    }

    var focusedType: TpslType? {
        switch focusField {
        case .takeProfit: .takeProfit
        case .stopLoss: .stopLoss
        case nil: nil
        }
    }

    func text(for field: AutocloseScene.Field) -> String {
        switch field {
        case .takeProfit: takeProfit.text
        case .stopLoss: stopLoss.text
        }
    }

    func field(
        type: TpslType,
        price: Double?,
        originalPrice: Double?,
        formattedPrice: String?,
        orderId: UInt64?,
    ) -> GemAutocloseField {
        let validator = type == .takeProfit ? takeProfitValidator : stopLossValidator
        return GemAutocloseField(
            tpslType: type.toGem(),
            price: price,
            originalPrice: originalPrice,
            formattedPrice: formattedPrice,
            validation: price.map { validator.validate(price: $0) } ?? .invalidAmount,
            orderId: orderId,
        )
    }

    func update() {
        takeProfit.update()
        stopLoss.update()
    }
}
