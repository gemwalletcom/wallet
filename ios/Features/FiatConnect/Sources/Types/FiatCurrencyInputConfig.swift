// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemNumberFormat
import Primitives
import SwiftUI

struct FiatCurrencyInputConfig: CurrencyInputConfigurable {
    let secondaryText: String
    let currencySymbol: String
    let numberFormat: GemNumberFormat

    var currencyPosition: CurrencyTextField.CurrencyPosition {
        .leading
    }

    var placeholder: String {
        .zero
    }

    var keyboardType: UIKeyboardType {
        .numberPad
    }

    var sanitizer: ((String) -> String)? {
        { numberFormat.sanitize(input: $0, maximumFractionDigits: nil, maximumIntegerDigits: nil) }
    }

    var actionStyle: CurrencyInputActionStyle?
    let onTapActionButton: VoidAction = nil
}
