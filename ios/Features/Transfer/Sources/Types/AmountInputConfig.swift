// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

struct AmountInputConfig: CurrencyInputConfigurable {
    let sceneType: AmountType
    let canSwitchInputType: Bool
    let inputType: AmountInputType
    let asset: Asset
    let currencyFormatter: CurrencyFormatter
    let numberSanitizer: NumberSanitizer
    let secondaryText: String
    let onTapActionButton: (() -> Void)?

    var placeholder: String {
        .zero
    }

    private var usesWholeAmounts: Bool {
        StakeChain(rawValue: asset.chain.rawValue)?.usesWholeAmounts ?? false
    }

    var keyboardType: UIKeyboardType {
        switch sceneType {
        case .transfer, .deposit, .withdraw, .perpetual, .earn: .decimalPad
        case let .stake(stakeType):
            switch stakeType {
            case .stake, .unstake: usesWholeAmounts ? .numberPad : .decimalPad
            case .redelegate, .withdraw, .claimRewards, .freeze, .unfreeze: .decimalPad
            }
        }
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

    var sanitizer: ((String) -> String)? {
        { numberSanitizer.sanitize($0) }
    }
}
