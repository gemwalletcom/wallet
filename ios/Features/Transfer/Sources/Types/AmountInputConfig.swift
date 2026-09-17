// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import enum Gemstone.GemAmountInputType
import struct Gemstone.GemNumberFormat
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

struct AmountInputConfig: CurrencyInputConfigurable {
    let canSwitchInputType: Bool
    let inputType: GemAmountInputType
    let asset: Asset
    let currencyFormatter: CurrencyFormatter
    let numberFormat: GemNumberFormat
    let secondaryText: String
    let onTapActionButton: (() -> Void)?
    let usesWholeAmounts: Bool

    var placeholder: String {
        .zero
    }

    var keyboardType: UIKeyboardType {
        usesWholeAmounts ? .numberPad : .decimalPad
    }

    var currencyPosition: CurrencyTextField.CurrencyPosition {
        switch inputType {
        case .asset: .trailing
        case .fiat: .leading
        }
    }

    var currencySymbol: String {
        switch inputType {
        case .asset: asset.symbol
        case .fiat: currencyFormatter.symbol
        }
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
