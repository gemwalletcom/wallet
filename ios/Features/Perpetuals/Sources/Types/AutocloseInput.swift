// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemAutocloseViewState
import Primitives
import PrimitivesComponents

@MainActor
struct AutocloseInput {
    var takeProfit = InputValidationViewModel(mode: .manual)
    var stopLoss = InputValidationViewModel(mode: .manual)
    var focusField: AutocloseScene.Field?

    init(takeProfitText: String?, stopLossText: String?) {
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

    func update(state: GemAutocloseViewState) {
        takeProfit.update(error: AutocloseViewModel(state: state.takeProfit).error)
        stopLoss.update(error: AutocloseViewModel(state: state.stopLoss).error)
    }
}
