// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAmountField
import struct Gemstone.GemNumberFormat
import Primitives
import PrimitivesComponents
import SwiftUI

struct FiatCurrencyInputConfig: CurrencyInputConfigurable {
    let secondaryText: String
    let field: GemAmountField
    let numberFormat: GemNumberFormat

    var currencyPosition: CurrencyTextField.CurrencyPosition {
        field.placement.position
    }

    var currencySymbol: String {
        field.symbol.text
    }

    var placeholder: String {
        .zero
    }

    var keyboardType: UIKeyboardType {
        field.keyboard.keyboardType
    }

    func sanitize(_ text: String) -> String {
        numberFormat.sanitize(input: text, maximumFractionDigits: 0, maximumIntegerDigits: nil)
    }

    var actionStyle: CurrencyInputActionStyle?
    let onTapActionButton: VoidAction = nil
}
