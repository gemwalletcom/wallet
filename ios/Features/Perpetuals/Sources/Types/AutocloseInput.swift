// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents

@MainActor
struct AutocloseInput {
    var takeProfit: InputValidationViewModel
    var stopLoss: InputValidationViewModel
    var focusField: AutocloseScene.Field?

    init(type: AutocloseType, takeProfitText: String?, stopLossText: String?) {
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

    func update() {
        takeProfit.update()
        stopLoss.update()
    }
}
