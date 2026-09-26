// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemPriceAlertInput
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import SwiftUI

struct SetPriceAlertCurrencyInputConfig: CurrencyInputConfigurable {
    let input: GemPriceAlertInput
    let secondaryText: String
    let formatter: CurrencyFormatter
    let onTapActionButton: VoidAction

    var placeholder: String {
        input.placeholder
    }

    var currencySymbol: String {
        switch input.symbol {
        case .currency: formatter.symbol
        case .percent: "%"
        }
    }

    var currencyPosition: Components.CurrencyTextField.CurrencyPosition {
        switch input.placement {
        case .leading: .leading
        case .trailing: .trailing
        }
    }

    var keyboardType: UIKeyboardType {
        .decimalPad
    }

    func sanitize(_ text: String) -> String {
        text
    }

    var actionStyle: CurrencyInputActionStyle? {
        input.directionButton.map {
            CurrencyInputActionStyle(
                position: .amount,
                image: $0.image,
            )
        }
    }
}
