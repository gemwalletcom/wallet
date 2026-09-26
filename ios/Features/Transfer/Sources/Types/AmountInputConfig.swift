// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAmountField
import struct Gemstone.GemNumberFormat
import GemstonePrimitives
import PrimitivesComponents
import Style
import SwiftUI

struct AmountInputConfig: CurrencyInputConfigurable {
    let field: GemAmountField
    let canSwitchInputType: Bool
    let numberFormat: GemNumberFormat
    let secondaryText: String
    let onTapActionButton: (() -> Void)?

    var placeholder: String {
        .zero
    }

    var keyboardType: UIKeyboardType {
        field.keyboard.keyboardType
    }

    var currencyPosition: CurrencyTextField.CurrencyPosition {
        field.placement.position
    }

    var currencySymbol: String {
        field.symbol.text
    }

    var actionStyle: CurrencyInputActionStyle? {
        guard canSwitchInputType else { return nil }
        return CurrencyInputActionStyle(
            position: .secondary,
            image: Images.Actions.swap.renderingMode(.template),
        )
    }

    func sanitize(_ text: String) -> String {
        numberFormat.sanitize(input: text, maximumFractionDigits: nil, maximumIntegerDigits: nil)
    }
}
