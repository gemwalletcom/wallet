// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

struct LightningAmountInputConfig: CurrencyInputConfigurable {
    static let satsSymbol = "sats"
    private static let maximumDigits = 12

    let secondaryText: String

    var placeholder: String {
        .zero
    }

    var currencySymbol: String {
        Self.satsSymbol
    }

    var currencyPosition: CurrencyTextField.CurrencyPosition {
        .trailing
    }

    var keyboardType: UIKeyboardType {
        .numberPad
    }

    var actionStyle: CurrencyInputActionStyle? {
        nil
    }

    var onTapActionButton: (() -> Void)? {
        nil
    }

    func sanitize(_ text: String) -> String {
        String(text.filter(\.isNumber).prefix(Self.maximumDigits))
    }
}
