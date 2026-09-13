// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Primitives
import SwiftUI

struct FiatCurrencyInputConfig: CurrencyInputConfigurable {
    let secondaryText: String
    let currencySymbol: String
    let numberSanitizer: NumberSanitizer

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
        { numberSanitizer.sanitize($0) }
    }

    var actionStyle: CurrencyInputActionStyle?
    let onTapActionButton: VoidAction = nil
}
