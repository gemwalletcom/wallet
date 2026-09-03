// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAutocloseField
import Primitives
import PrimitivesComponents
import Validators

@MainActor
struct AutocloseInput {
    var takeProfit: InputValidationViewModel
    var stopLoss: InputValidationViewModel
    var focusField: AutocloseScene.Field?

    init(type: AutocloseType, takeProfitText: String?, stopLossText: String?) {
        takeProfit = InputValidationViewModel(
            mode: .manual,
            validators: [AutocloseValidator(type: .takeProfit, direction: type.direction, marketPrice: type.marketPrice)],
        )
        stopLoss = InputValidationViewModel(
            mode: .manual,
            validators: [AutocloseValidator(type: .stopLoss, direction: type.direction, marketPrice: type.marketPrice)],
        )

        takeProfitText.map { takeProfit.text = $0 }
        stopLossText.map { stopLoss.text = $0 }
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
        let input = type == .takeProfit ? takeProfit : stopLoss
        return GemAutocloseField(
            price: price,
            originalPrice: originalPrice,
            formattedPrice: formattedPrice,
            isValid: price != nil && input.isValid,
            orderId: orderId,
        )
    }

    func update() {
        takeProfit.update()
        stopLoss.update()
    }
}
